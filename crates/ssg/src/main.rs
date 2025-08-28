use std::{
    collections::HashSet,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};
use clap::Parser;
use glob::glob;
use ssg_generator_utils::{generate_site, load_meta};
use syntect::parsing::SyntaxSet;
use tailwindcss_oxide::scanner::{Scanner, sources::PublicSourceEntry};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Deserialize, Serialize, Default)]
#[command(author, version, about = "Static site generator", long_about = None)]
struct Config {
    /// Content source directory
    #[arg(long)]
    base: Option<String>,

    /// Templates directory
    #[arg(long)]
    templates: Option<String>,

    /// Output directory
    #[arg(long)]
    dist: Option<String>,

    /// Base domain for sitemap URLs (e.g., https://example.com)
    #[arg(long)]
    domain: Option<String>,

    /// Base path for sitemap URLs (e.g., /blog)
    #[arg(long)]
    base_path: Option<String>,

    /// Path to a JSON configuration file
    #[arg(long)]
    config: Option<String>,

    /// Dump syntaxes and exit
    #[arg(long)]
    #[serde(default)]
    dump: bool,

    /// Comma-separated list of languages to omit from syntax highlighting
    #[arg(long)]
    omit_languages: Option<String>,

    /// Disable syntax highlighting altogether
    #[arg(long)]
    #[serde(default)]
    no_syntax_highlighting: bool,
}

impl Config {
    fn merge(self, other: Self) -> Self {
        Self {
            base: self.base.or(other.base),
            templates: self.templates.or(other.templates),
            dist: self.dist.or(other.dist),
            domain: self.domain.or(other.domain),
            base_path: self.base_path.or(other.base_path),
            config: self.config.or(other.config),
            dump: self.dump || other.dump,
            omit_languages: self.omit_languages.or(other.omit_languages),
            no_syntax_highlighting: self.no_syntax_highlighting || other.no_syntax_highlighting,
        }
    }
}

fn get_md_files(base_path: &Path) -> Vec<PathBuf> {
    let pattern = base_path.join("**/*.md").to_string_lossy().to_string();
    glob(&pattern)
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok)
        .collect()
}

fn dump_syntaxes() {
    let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
    builder
        .add_from_folder("crates/ssg-generator-utils/syntaxes", true)
        .expect("Failed to load syntaxes");
    let ps = builder.build();

    let file = File::create("crates/ssg-generator-utils/syntaxes/syntaxes.packdump").unwrap();
    syntect::dumps::dump_to_writer(&ps, &file).unwrap();
    println!("SyntaxSet dumped to syntaxes.packdump");

    let mut list_file = File::create("syntaxes_supported.txt").unwrap();
    for syntax in ps.syntaxes() {
        writeln!(list_file, "{}", syntax.name).unwrap();
    }
    println!("Supported syntaxes list saved to syntaxes_supported.txt");
}

/// Entrypoint for the CLI: generate a static site or dump editor syntaxes.
///
/// Parses CLI arguments and either:
/// - when `--dump` is set: dumps bundled syntaxes and exits; or
/// - otherwise: generates the site from Markdown under the configured `base` directory into `dist`,
///   loading metadata from `base/meta.yml` and passing optional `llm_title` and `llm_description` into the generator.
/// The function also creates the `dist` directory if missing, writes a space-separated `candidates.txt` of scanned HTML files,
/// and prints progress/errors to stdout/stderr.
///
/// Notes:
/// - The CLI `domain` value should include the protocol and a trailing slash (e.g. `https://example.com/`).
/// - This function performs filesystem IO (creates directories, writes files) and may print error messages instead of panicking.
///
/// # Examples
///
/// ```no_run
/// // Run the program as a binary; example shows typical CLI invocation.
/// // $ my_ssg --base pages --dist dist --domain https://example.com/
/// std::env::set_var("RUST_BACKTRACE", "0");
/// // `main()` is the process entrypoint and will perform filesystem operations when run.
/// crate::main();
/// ```
fn main() {
    let cli_config = Config::parse();

    let config_path = cli_config.config.as_deref().unwrap_or("cats-ssg.json");
    let file_config: Config = if Path::new(config_path).exists() {
        let file = File::open(config_path).expect("Failed to open config file");
        serde_json::from_reader(file).expect("Failed to parse config file")
    } else {
        Config::default()
    };

    let config = cli_config.merge(file_config);

    if config.dump {
        dump_syntaxes();
        return;
    }

    let base = Path::new(config.base.as_deref().unwrap_or("pages"));
    let templates_path = Path::new(config.templates.as_deref().unwrap_or("templates"));
    let dist = Path::new(config.dist.as_deref().unwrap_or("dist"));
    let domain = config.domain.as_deref().unwrap_or("https://shadowrunner8095.github.io/my-blog/");
    let base_path = config.base_path.as_deref().unwrap_or("");

    if !dist.exists() {
        fs::create_dir_all(dist).unwrap();
    }
    let md_files = get_md_files(base);

    let content_index_path = Path::new("crates/ssg-generator-utils/content-index.html");
    let main_meta_inf = load_meta(&base.join("meta.yml"));

    let llms_title = main_meta_inf.llm_title.as_deref();
    let llms_description = main_meta_inf.llm_description.as_deref();

    let omit_languages: HashSet<String> = match config.omit_languages {
        Some(langs) => langs
            .split(',')
            .map(String::from)
            .filter(|s| !s.is_empty())
            .collect(),
        None => {
            let mut default = HashSet::new();
            default.insert("mermaid".to_string());
            default
        }
    };

    if let Err(e) = generate_site(
        md_files,
        base,
        dist,
        domain,
        base_path,
        templates_path,
        content_index_path,
        Some(true),
        llms_title,
        llms_description,
        &omit_languages,
        config.no_syntax_highlighting,
    ) {
        eprintln!("Failed to generate site: {}", e);
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
