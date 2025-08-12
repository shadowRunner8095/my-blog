use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};
use clap::Parser;
use glob::glob;
use ssg_generator_utils::{generate_site, load_meta};
use syntect::parsing::SyntaxSet;
use tailwindcss_oxide::scanner::{Scanner, sources::PublicSourceEntry};

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
    let cli = Cli::parse();

    if cli.dump {
        dump_syntaxes();
        return;
    }

    let base = Path::new(&cli.base);
    let dist = Path::new(&cli.dist);
    if !dist.exists() {
        fs::create_dir_all(dist).unwrap();
    }
    let md_files = get_md_files(base);

    let templates_path = Path::new("crates/ssg-generator-utils/templates");
    let syntaxes_path = Path::new("crates/ssg-generator-utils/syntaxes");
    let content_index_path = Path::new("crates/ssg-generator-utils/content-index.html");
    let main_meta_inf = load_meta(&base.join("meta.yml"));

    let llms_title = main_meta_inf.llm_title.as_deref();
    let llms_description = main_meta_inf.llm_description.as_deref();

    if let Err(e) = generate_site(
        md_files,
        base,
        dist,
        &cli.domain,
        templates_path,
        syntaxes_path,
        content_index_path,
        Some(true),
        llms_title,
        llms_description,
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
