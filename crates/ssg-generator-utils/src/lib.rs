#![warn(unused_extern_crates)]
use std::{
    fs::{self, File},
    io::{BufReader},
    path::{Path, PathBuf},
};
use rayon::prelude::*;
use pulldown_cmark::{Parser as MdParser, Options, html, Event, Tag, CodeBlockKind, TagEnd};
use syntect::{parsing::SyntaxSet, highlighting::ThemeSet, html::highlighted_html_for_string};
use serde::Deserialize;
use minijinja::{Environment, context};

pub mod sitemap;

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Meta {
    pub title: Option<String>,
    extends: Option<String>,
    generate_llm_txt: Option<bool>,
    omit_llm_txt_generation: Option<bool>,
    description: Option<String>,
    pub llm_description: Option<String>,
    keywords: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    merge_tags_keywords: Option<bool>,
    page_slug: Option<String>,
    pub llm_title: Option<String>
}

/// Load metadata from a YAML file into a `Meta` struct.
///
/// Attempts to read and deserialize YAML from `meta_path`. If the file cannot be read
/// or the YAML fails to deserialize, returns `Meta::default()`.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// let meta = load_meta(Path::new("content/page/meta.yml"));
/// ```
pub fn load_meta(meta_path: &Path) -> Meta {
    if let Ok(content) = fs::read_to_string(meta_path) {
        serde_yaml::from_str(&content).unwrap_or_default()
    } else {
        Meta::default()
    }
}

use regex::Regex;

/// Removes all occurrences of an HTML-like tag and its contents (including the tags).
///
/// The removal is case-insensitive and matches across newlines; attributes on the opening
/// tag are allowed and removed together with the enclosed content.
///
/// # Examples
///
/// ```
/// let s = "<p>keep</p><secret attr=\"x\">remove\nthis</secret><div>ok</div>";
/// let out = remove_tag_and_contents(s, "secret");
/// assert_eq!(out, "<p>keep</p><div>ok</div>");
/// ```
pub fn remove_tag_and_contents(md: &str, tag: &str) -> String {
    let pattern = format!(r"(?is)<{0}[^>]*?>.*?</{0}>", regex::escape(tag));
    let re = Regex::new(&pattern).unwrap();
    re.replace_all(md, "").to_string()
}

/// Removes only the specified HTML-like opening and closing tags from `md`, preserving the inner content.
///
/// The tag match is case-insensitive and will remove any attributes on the opening tag (e.g. `<tag attr="x">`).
/// Only the exact opening and matching closing tags are removed; content between them remains unchanged.
///
/// # Examples
///
/// ```
/// let s = "<only-in-llm-txt>Keep this text</only-in-llm-txt>";
/// let out = remove_tag_only(s, "only-in-llm-txt");
/// assert_eq!(out, "Keep this text");
/// ```
pub fn remove_tag_only(md: &str, tag: &str) -> String {
    let open = format!(r"(?i)<{0}[^>]*?>", regex::escape(tag));
    let close = format!(r"(?i)</{0}>", regex::escape(tag));
    let re_open = Regex::new(&open).unwrap();
    let re_close = Regex::new(&close).unwrap();
    let result = re_open.replace_all(md, "");
    re_close.replace_all(&result, "").to_string()
}

// Example usage before parsing:
// let md = remove_tag_and_contents(md, "ignore-content");
// let md = remove_tag_only(md, "ignore-content");
/// Converts a folder Path to a human-readable title by splitting on '-' and capitalizing each segment.
///
/// If the path has no file name or cannot be converted to UTF-8, returns "Untitled".
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// assert_eq!(crate::folder_name_to_title(Path::new("my-folder-name")), "My Folder Name");
/// assert_eq!(crate::folder_name_to_title(Path::new("single")), "Single");
/// assert_eq!(crate::folder_name_to_title(Path::new("")), "Untitled");
/// ```
fn folder_name_to_title(folder: &Path) -> String {
    folder
        .file_name()
        .and_then(|os| os.to_str())
        .map(|name| {
            name.split('-')
                .map(|w| {
                    let mut c = w.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_else(|| "Untitled".to_string())
}

fn markdown_to_html(md: &str, ps: &SyntaxSet, theme: &syntect::highlighting::Theme) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = MdParser::new_ext(md, options);
    let mut html_output = String::new();
    let mut in_code_block = false;
    let mut code_lang = None;
    let mut code_content = String::new();
    let mut events = Vec::new();

    for event in parser {
        match &event {
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                code_lang = match kind {
                    CodeBlockKind::Fenced(lang) => Some(lang.to_string()),
                    _ => None,
                };
                code_content.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                let syntax = code_lang
                    .as_deref()
                    .and_then(|lang| ps.find_syntax_by_token(lang))
                    .unwrap_or_else(|| ps.find_syntax_plain_text());
                let mut highlighted =
                    highlighted_html_for_string(&code_content, ps, syntax, theme).unwrap();
                if highlighted.ends_with('\n') {
                    highlighted.pop();
                }
                events.push(Event::Html(highlighted.into()));
                in_code_block = false;
            }
            Event::Text(text) if in_code_block => {
                code_content.push_str(&text);
            }
            _ if in_code_block => {}
            _ => events.push(event),
        }
    }

    html::push_html(&mut html_output, events.into_iter());
    html_output
}

/// Processes a single Markdown source file into an HTML page, optionally writes a stripped Markdown copy for LLM use, and returns metadata for site indexing.
///
/// This function:
/// - Reads the Markdown file at `src_path` and loads per-file metadata from a sibling `meta.yml`.
/// - Determines the page title (from metadata, index folder name, or file stem).
/// - Preprocesses the Markdown to remove or preserve LLM-specific tags:
///   - `<exclude-from-llm-txt>`: kept for HTML generation but removed from any copied Markdown for LLM consumption.
///   - `<only-in-llm-txt>`: removed (and its contents removed) before HTML generation; also removed from the final rendered HTML.
/// - Converts the sanitized Markdown to HTML with `markdown_to_html`, renders it with the configured template (default `"base.html"`), and writes the resulting HTML under `dist_path` mirroring `base_path` (with special handling for `index.md` + `page_slug`).
/// - Optionally writes a stripped copy of the Markdown next to the generated HTML (controlled by metadata fields `omit_llm_txt_generation`, `generate_llm_txt`, or the `generate_llm_txt_by_default` argument).
/// - Returns None on I/O or template errors; on success returns a tuple:
///   (title, href_for_sitemap, optional_relative_md_path_if_copied, optional_llm_description_from_meta, md_was_copied_flag).
///
/// Notes:
/// - Side effects: creates directories, writes HTML files, and may write a stripped Markdown file.
/// - Returns `None` if reading the source, creating directories, or writing output fails.
/// - The returned `href_for_sitemap` is a path prefixed with `/my-blog/` suitable for sitemap/index entries.
///
/// # Examples
///
/// ```ignore
/// // Example (non-compiling stub): call with appropriate SyntaxSet, Theme and Minijinja Environment.
/// let result = process_md_file(src_path, base_path, dist_path, &ps, &theme, &env, Some(true));
/// if let Some((title, href, md_rel, llm_desc, copied)) = result {
///     println!("Generated {} -> {}, md copied: {}", title, href, copied);
/// }
/// ```
fn process_md_file(
    src_path: &Path,
    base_path: &Path,
    dist_path: &Path,
    ps: &SyntaxSet,
    theme: &syntect::highlighting::Theme,
    env: &Environment,
    generate_llm_txt_by_default: Option<bool>,
) -> Option<(String, String, Option<String>, Option<String>, bool)> {
    let md_content = match fs::read_to_string(src_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read {}: {}", src_path.display(), e);
            return None;
        }
    };

    let meta_path = src_path.with_file_name("meta.yml");
    let meta = load_meta(&meta_path);

    let title = meta.title.clone().unwrap_or_else(|| {
        if src_path.file_name().map_or(false, |f| f == "index.md") {
            folder_name_to_title(src_path.parent().unwrap_or_else(|| Path::new("")))
        } else {
            src_path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Untitled".to_string())
        }
    });


    // Remove <exclude-from-llm-txt> tags (but keep their content) before HTML generation
    let md_content_no_exclude_tag = remove_tag_only(&md_content, "exclude-from-llm-txt");
    // Remove <only-in-llm-txt> tags AND their content before HTML generation
    let md_content_no_tags = remove_tag_and_contents(&md_content_no_exclude_tag, "only-in-llm-txt");
    let body_html = markdown_to_html(&md_content_no_tags, ps, theme);

    let template_name = meta.extends.as_deref().unwrap_or("base.html");
    let rendered = if let Some(tmpl) = env.get_template(template_name).ok() {
        tmpl.render(context! {
            title => &title,
            body => &body_html,
        })
        .unwrap_or_else(|e| {
            eprintln!("Template render error for {}: {}", src_path.display(), e);
            body_html.clone()
        })
    } else {
        eprintln!("Template {} not found, rendering body only for {}", template_name, src_path.display());
        body_html.clone()
    };

    let rel_path = src_path.strip_prefix(base_path).unwrap();
    let mut dest_path = dist_path.join(rel_path);

    // If this is index.md and meta.page_slug is set, use it as the last directory
    if src_path.file_name().map_or(false, |f| f == "index.md") {
        if let Some(ref slug) = meta.page_slug {
            // Replace the last component (parent dir) with the slug
            if let Some(parent) = dest_path.parent().and_then(|p| p.parent()) {
                dest_path = parent.join(slug).join("index.html");
            } else {
                dest_path = dist_path.join(slug).join("index.html");
            }
        } else {
            dest_path.set_extension("html");
        }
    } else {
        dest_path.set_extension("html");
    }

    if let Some(parent) = dest_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("Failed to create directory {}: {}", parent.display(), e);
            return None;
        }
    }


    // After HTML generation, remove <only-in-llm-txt> and its content from the HTML
    let rendered_final = remove_tag_and_contents(&rendered, "only-in-llm-txt");
    if let Err(e) = fs::write(&dest_path, &rendered_final) {
        eprintln!("Failed to write {}: {}", dest_path.display(), e);
        return None;
    }

 
    let should_copy_md = if meta.omit_llm_txt_generation.unwrap_or(false) {
        false
    } else if let Some(val) = meta.generate_llm_txt {
        val
    } else {
        generate_llm_txt_by_default.unwrap_or(false)
    };

    let mut md_rel_path: Option<String> = None;
    let mut md_copied = false;
    if should_copy_md {
        if let Some(parent) = dest_path.parent() {
            let md_filename = src_path.file_name().unwrap();
            let md_dest = parent.join(md_filename);
            // Write the stripped md content (with <exclude-from-llm-txt> tag and its content removed, and <only-in-llm-txt> tag only removed)
            let md_content_no_exclude = remove_tag_and_contents(&md_content, "exclude-from-llm-txt");
            let md_content_no_only_tag = remove_tag_only(&md_content_no_exclude, "only-in-llm-txt");
            if let Err(e) = fs::write(&md_dest, &md_content_no_only_tag) {
                eprintln!("Failed to write stripped markdown file to {}: {}", md_dest.display(), e);
            } else {
                // Compute relative path from dist_path
                if let Ok(rel_md) = md_dest.strip_prefix(dist_path) {
                    md_rel_path = Some(rel_md.to_string_lossy().replace('\\', "/"));
                    md_copied = true;
                }
            }
        }
    }

    // Compute href for sitemap/index
    let href = if src_path.file_name().map_or(false, |f| f == "index.md") {
        if let Some(ref slug) = meta.page_slug {
            format!("/my-blog/{}/index.html", slug)
        } else {
            format!(
                "/my-blog/{}",
                rel_path.with_extension("html").to_string_lossy().replace('\\', "/")
            )
        }
    } else {
        format!(
            "/my-blog/{}",
            rel_path.with_extension("html").to_string_lossy().replace('\\', "/")
        )
    };

    Some((title, href, md_rel_path, meta.llm_description.clone(), md_copied))
}

/// Create a "content-index" page under `dist_path` using the template at `content_index_path`.
///
/// Reads the template file, registers it in the provided Minijinja `env` as `"content-index.html"`,
/// renders it with `entries` mapped to `{ pages: [{ title, href }, ...], title: "Index Content" }`,
/// and writes the result to `<dist_path>/content-index/index.html`.
///
/// `entries` must be a slice of `(title, href)` pairs; any leading "/my-blog/" prefix in `href` is
/// stripped before rendering so links in the index are relative to the site root.
///
/// Errors from file I/O or template rendering are propagated via the `Result`.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use minijinja::Environment;
/// # // Assume `create_index_page` is available in this crate.
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let dist = Path::new("dist");
/// let content_index_template = Path::new("templates/content-index.html");
/// let mut env = Environment::new();
///
/// let entries = vec![
///     ("First Page".to_string(), "/my-blog/first/index.html".to_string()),
///     ("Second Page".to_string(), "second.html".to_string()),
/// ];
///
/// create_index_page(dist, &entries, &mut env, content_index_template)?;
/// # Ok(()) }
/// ```
fn create_index_page(
    dist_path: &Path,
    entries: &[(String, String)],
    env: &mut Environment,
    content_index_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let index_template_str = fs::read_to_string(content_index_path)?;

    env.add_template_owned("content-index.html", index_template_str)?;

    let items: Vec<_> = entries
        .iter()
        .map(|(title, href)| {
            let href = href.strip_prefix("/my-blog/").unwrap_or(href).to_string();
            context! { href => href, title => title.clone() }
        })
        .collect();

    let rendered = env.get_template("content-index.html")?.render(context! {
        pages => items,
        title => "Index Content",
    })?;

    let index_dir = dist_path.join("content-index");
    fs::create_dir_all(&index_dir)?;
    let index_path = index_dir.join("index.html");
    fs::write(index_path, rendered)?;

    Ok(())
}

/// Generate a static site from a list of Markdown files, write supporting artifacts, and return metadata.
///
/// Processes the provided Markdown files (in parallel) to produce HTML pages under `dist_path` using
/// templates from `templates_path` and syntax definitions from `syntaxes_path`. Side effects:
/// - Writes generated HTML files (and optional stripped Markdown copies) into `dist_path`.
/// - Writes `sitemap.xml` to `dist_path`.
/// - Creates a content index page at `{dist_path}/content-index/index.html` using `content_index_path`.
/// - Writes `llms.txt` to `dist_path` listing pages whose Markdown was copied for LLM consumption.
///
/// Behavior notes:
/// - Syntax highlighting is loaded from `syntaxes_path/syntaxes.packdump` and a default dark theme is used.
/// - Template loader is rooted at `templates_path`; missing templates fall back to body HTML for that page.
/// - The `generate_llm_txt_by_default` flag determines the default behavior for copying stripped Markdown files:
///   meta flags on a per-file basis (generate_llm_txt, omit_llm_txt_generation) override this default.
/// - `llms_title` and `llms_description`, if provided, are used as the header in `llms.txt`.
///
/// Returns:
/// - Ok((entries, md_paths)) where:
///   - `entries` is a Vec of (title, href, md_rel_path, llm_description) for all processed pages (md_rel_path and llm_description may be None).
///   - `md_paths` is a Vec of relative paths (strings) of Markdown files that were copied for LLM use.
/// - Err(...) if an early fatal error occurs (e.g., failing to read syntax data or other IO/parsing errors during initialization).
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// // Call with no markdown files; this will initialize and produce empty outputs in the temp dir.
/// let md_files: Vec<PathBuf> = Vec::new();
/// let base = std::env::temp_dir();
/// let dist = std::env::temp_dir();
/// let templates = std::env::temp_dir();
/// let syntaxes = std::env::temp_dir();
/// let content_index = std::env::temp_dir();
/// let res = crate::generate_site(
///     md_files,
///     &base,
///     &dist,
///     "https://example.com",
///     &templates,
///     &syntaxes,
///     &content_index,
///     Some(false),
///     None,
///     None,
/// );
/// assert!(res.is_ok());
/// ```
pub fn generate_site(
    md_files: Vec<PathBuf>,
    base_path: &Path,
    dist_path: &Path,
    domain: &str,
    templates_path: &Path,
    syntaxes_path: &Path,
    content_index_path: &Path,
    generate_llm_txt_by_default: Option<bool>,
    llms_title: Option<&str>,
    llms_description: Option<&str>,
) -> Result<(Vec<(String, String, Option<String>, Option<String>)>, Vec<String>), Box<dyn std::error::Error>> {
    let file = File::open(syntaxes_path.join("syntaxes.packdump"))?;
    let reader = BufReader::new(file);
    let ps: SyntaxSet = syntect::dumps::from_reader(reader)?;

    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"];

    let mut env = Environment::new();
    env.set_loader(minijinja::path_loader(templates_path));

    let domain = domain.trim_end_matches('/');
    let sitemap_urls: Vec<String> = md_files.iter().map(|p| {
        let rel = p.strip_prefix(base_path).unwrap();
        let url = rel.with_extension("html").to_string_lossy().replace('\\', "/");
        format!("{}/{}", domain, url)
    }).collect();

    let sitemap_refs: Vec<&str> = sitemap_urls.iter().map(|s| s.as_str()).collect();
    let sitemap_path = dist_path.join("sitemap.xml");

    let results: Vec<_> = md_files
        .par_iter()
        .filter_map(|file| process_md_file(file, base_path, dist_path, &ps, theme, &env, generate_llm_txt_by_default))
        .collect();
    let entries: Vec<_> = results.iter().map(|(title, href, _, _, _)| (title.clone(), href.clone())).collect();
    let md_paths: Vec<String> = results.iter().filter_map(|(_, _, md, _, md_copied)| if *md_copied { md.clone() } else { None }).collect();

    println!("Processed all markdown files.");
    if let Err(e) = sitemap::write_sitemap(&sitemap_refs, sitemap_path.to_string_lossy().as_ref()) {
        eprintln!("Failed to write sitemap: {}", e);
    }
    if let Err(e) = create_index_page(dist_path, &entries, &mut env, content_index_path) {
        eprintln!("Failed to create index page: {}", e);
    } else {
        println!("Index page generated at {}/content-index/index.html", dist_path.display());
    }

    use std::fmt::Write as _;
    let mut llms_tx = String::new();
    let llms_title = llms_title.unwrap_or("LLM Content Index");
    let llms_description = llms_description.unwrap_or("");
    writeln!(llms_tx, "# {}\n", llms_title).ok();
    if !llms_description.trim().is_empty() {
        writeln!(llms_tx, "{}\n", llms_description.trim()).ok();
    }
    writeln!(llms_tx, "## Contents\n").ok();
    for (title, _href, md, llm_description, md_copied) in &results {

        if !md_copied { continue; }
        // Remove any leading "/my-blog" or similar base path from href before joining with domain
 
        if let Some(md_path) = md {
            writeln!(llms_tx, "- [{}]({}){}",
                title,
                format!("{}/{}", domain, md_path),
                match llm_description {
                    Some(desc) if !desc.trim().is_empty() => format!(": {}", desc.trim()),
                    _ => String::new(),
                }
            ).ok();
        }
    }
    let llms_tx_path = dist_path.join("llms.txt");
    if let Err(e) = std::fs::write(&llms_tx_path, llms_tx) {
        eprintln!("Failed to write llms.tx: {}", e);
    } else {
        println!("llms.tx generated at {}", llms_tx_path.display());
    }
    // Remove md_copied from results in return value for compatibility
    Ok((results.into_iter().map(|(a,b,c,d,_e)| (a,b,c,d)).collect(), md_paths))
}
