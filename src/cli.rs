use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum OutputFormatArg {
    Json,
    Csv,
}

#[derive(Debug, Parser, Clone)]
#[command(
    name = "rust_crawler",
    version,
    about = "A multi-threaded Rust web crawler course project"
)]
pub struct Cli {
    #[arg(short, long, required = true, num_args = 1.., value_delimiter = ',')]
    pub start_urls: Vec<String>,

    #[arg(short = 'd', long, default_value_t = 2)]
    pub max_depth: usize,

    #[arg(short = 'm', long, default_value_t = 20)]
    pub max_pages: usize,

    #[arg(short = 'w', long, default_value_t = 4)]
    pub worker_count: usize,

    #[arg(long, default_value_t = 1)]
    pub max_retries: usize,

    #[arg(short, long, default_value = "output/result.json")]
    pub output: PathBuf,

    #[arg(long, value_enum)]
    pub format: Option<OutputFormatArg>,

    #[arg(long)]
    pub report: bool,

    #[arg(long)]
    pub report_output: Option<PathBuf>,

    #[arg(long)]
    pub title_keyword: Option<String>,

    #[arg(long)]
    pub url_keyword: Option<String>,

    #[arg(long)]
    pub min_content_length: Option<usize>,

    #[arg(long, default_value_t = false)]
    pub success_only: bool,

    #[arg(long, default_value_t = true)]
    pub same_domain_only: bool,

    #[arg(long, default_value_t = 10)]
    pub timeout_secs: u64,
}

impl Cli {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}
