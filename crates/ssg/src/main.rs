use std::{
    fs::{self, File},
    io::{BufReader, Write},
    path::{Path, PathBuf}
};
use clap::Parser;
#[derive(Parser, Debug)]
#[command(author, version, about = "Static site generator", long_about = None)]
struct Cli {
    /// Content source directory
    #[arg(long, default_value = "pages")]
    base: String,

    /// Output directory
    #[arg(long, default_value = "dist")]
    dist: String,

    /// Base domain for sitemap URLs (should include protocol and trailing slash)
    #[arg(long, default_value = "https://shadowrunner8095.github.io/my-blog/")]
    domain: String,

    /// Dump syntaxes and exit
    #[arg(long)]
    dump: bool,
}

use glob::glob;
use rayon::prelude::*;
use pulldown_cmark::{Parser as MdParser, Options, html, Event, Tag, CodeBlockKind, TagEnd};
use syntect::{parsing::SyntaxSet, highlighting::ThemeSet, html::highlighted_html_for_string};
use serde::Deserialize;
use minijinja::{Environment, context};
use tailwindcss_oxide::scanner::{Scanner, sources::PublicSourceEntry};

use crate::sitemap::write_sitemap;

#[derive(Deserialize, Debug, Default, Clone)]
struct Meta {
    title: Option<String>,
    extends: Option<String>,
}

fn load_meta(meta_path: &Path) -> Meta {
    if let Ok(content) = fs::read_to_string(meta_path) {
        serde_yaml::from_str(&content).unwrap_or_default()
    } else {
        Meta::default()
    }
}


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

fn get_md_files(base_path: &Path) -> Vec<PathBuf> {
    let pattern = base_path.join("**/*.md").to_string_lossy().to_string();
    glob(&pattern)
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok)
        .collect()
}

fn process_md_file(
    src_path: &Path,
    base_path: &Path,
    dist_path: &Path,
    ps: &SyntaxSet,
    theme: &syntect::highlighting::Theme,
    env: &Environment,
) -> Option<(String, String)> {
    let md_content = match fs::read_to_string(src_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read {}: {}", src_path.display(), e);
            return None;
        }
    };

    // Load meta.yml (same dir as MD)
    let meta_path = src_path.with_file_name("meta.yml");
    let meta = load_meta(&meta_path);

    // Compute title
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

    // Render markdown
    let body_html = markdown_to_html(&md_content, ps, theme);

    // Pick template
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

    // Write output
    let rel_path = src_path.strip_prefix(base_path).unwrap();
    let mut dest_path = dist_path.join(rel_path);
    dest_path.set_extension("html");
    if let Some(parent) = dest_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("Failed to create directory {}: {}", parent.display(), e);
            return None;
        }
    }
    if let Err(e) = fs::write(&dest_path, &rendered) {
        eprintln!("Failed to write {}: {}", dest_path.display(), e);
        return None;
    }

    // Return (title, href path relative to /my-blog root)
    let href = format!(
        "/my-blog/{}",
        rel_path.with_extension("html").to_string_lossy().replace('\\', "/")
    );

    Some((title, href))
}

fn dump_syntaxes() {
    let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
    builder
        .add_from_folder("crates/ssg/syntaxes", true)
        .expect("Failed to load syntaxes");
    let ps = builder.build();

    let file = File::create("syntaxes.packdump").unwrap();
    syntect::dumps::dump_to_writer(&ps, &file).unwrap();
    println!("SyntaxSet dumped to syntaxes.packdump");

    let mut list_file = File::create("syntaxes_supported.txt").unwrap();
    for syntax in ps.syntaxes() {
        writeln!(list_file, "{}", syntax.name).unwrap();
    }
    println!("Supported syntaxes list saved to syntaxes_supported.txt");
}

fn create_index_page(
    dist_path: &Path,
    entries: &[(String, String)],
    env: &mut Environment,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load the index template from root folder (not templates/)
    let index_template_str = fs::read_to_string("crates/ssg/content-index.html")?;

    env.add_template_owned("content-index.html", index_template_str)?;

    // Prepare vector of objects with href (no /my-blog/) and title
    let items: Vec<_> = entries
        .iter()
        .map(|(title, href)| {
            let href = href.strip_prefix("/my-blog/").unwrap_or(href).to_string();
            context! { href => href, title => title.clone() }
        })
        .collect();

    // Render with pages array and fixed title
    let rendered = env.get_template("content-index.html")?.render(context! {
        pages => items,
        title => "Index Content",
    })?;

    // Write to dist/content-index/index.html
    let index_dir = dist_path.join("content-index");
    fs::create_dir_all(&index_dir)?;
    let index_path = index_dir.join("index.html");
    fs::write(index_path, rendered)?;

    Ok(())
}
mod sitemap;


fn main() {
    let cli = Cli::parse();

    if cli.dump {
        dump_syntaxes();
        return;
    }

    let base = Path::new(&cli.base);
    let dist = Path::new(&cli.dist);
    let file = File::open("crates/ssg/syntaxes.packdump").expect("syntaxes.packdump not found — run with `--dump` first");
    let reader = BufReader::new(file);
    let ps: SyntaxSet = syntect::dumps::from_reader(reader).expect("Failed to read syntaxes.packdump");

    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"];

    let mut env = Environment::new();
    env.set_loader(minijinja::path_loader("crates/ssg/templates"));

    let md_files = get_md_files(base);
    println!("Found {} markdown files", md_files.len());

    //TODO: the sitemap needs a changefreq and priority tags also
    let domain = cli.domain.trim_end_matches('/');
    let sitemap_urls: Vec<String> = md_files.iter().map(|p| {
        let rel = p.strip_prefix(base).unwrap();
        let url = rel.with_extension("html").to_string_lossy().replace('\\', "/");
        format!("{}/{}", domain, url)
    }).collect();

    let sitemap_refs: Vec<&str> = sitemap_urls.iter().map(|s| s.as_str()).collect();
    let sitemap_path = dist.join("sitemap.xml");
    if let Err(e) = write_sitemap(&sitemap_refs, sitemap_path.to_string_lossy().as_ref()) {
        eprintln!("Failed to write sitemap: {}", e);
    }

    // Process all markdown files and collect title + href info for index page
    let entries: Vec<_> = md_files
        .par_iter()
        .filter_map(|file| process_md_file(file, base, dist, &ps, theme, &env))
        .collect();

    println!("Processed all markdown files.");

    if let Err(e) = create_index_page(dist, &entries, &mut env) {
        eprintln!("Failed to create index page: {}", e);
    } else {
        println!("Index page generated at {}/content-index/index.html", cli.dist);
    }

    let mut scanner = Scanner::new(vec![PublicSourceEntry{
        base: dist.to_string_lossy().to_string(),
        pattern: "**/*.html".into(),
        negated: false,
    }]);

    let candidates_path = dist.join("candidates.txt");
    if let Err(e) = fs::write(&candidates_path, scanner.scan().join(" ")) {
        eprintln!("Failed to write candidates.txt: {}", e);
    }

    println!("All done!");
}
