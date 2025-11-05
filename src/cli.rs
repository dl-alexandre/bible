use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "bible-static-generator")]
#[command(about = "Generate static HTML and JSON for Bible datasets", long_about = None)]
pub struct Cli {
    #[arg(long, help = "Path to dataset files (can be specified multiple times)")]
    pub datasets: Vec<PathBuf>,

    #[arg(long, default_value = "out", help = "Output directory")]
    pub out: PathBuf,

    #[arg(long, help = "Minify JSON output")]
    pub minify_json: bool,

    #[arg(long, help = "Compress JSON with gzip")]
    pub gzip_json: bool,

    #[arg(long, help = "Validate only (no generation)")]
    pub validate_only: bool,

    #[arg(long, default_value_t = 0.70, help = "Jaccard similarity threshold")]
    pub threshold_jaccard: f64,

    #[arg(long, default_value_t = 0.15, help = "Levenshtein distance threshold")]
    pub threshold_lev: f64,

    #[arg(long, help = "Use UTC timestamp (normalized)")]
    pub utc_timestamp: bool,

    #[arg(long, default_value = "1.0", help = "Schema version")]
    pub schema_version: String,

    #[arg(long, help = "Template directory")]
    pub templates: Option<PathBuf>,

    #[arg(long, help = "Log directory")]
    pub log_dir: Option<PathBuf>,

    #[arg(long, default_value = "https://example.com", help = "Base URL for sitemap (e.g., https://username.github.io/repo-name)")]
    pub base_url: String,
}

impl Cli {
    pub fn parse() -> Self {
        Parser::parse()
    }
}

