use clap::Parser;
use clap_verbosity_flag::{Verbosity, WarnLevel};

const DEFAULT_EXTENSION: &str = "webp";

#[derive(Parser, Debug)]
#[command(about)]
pub struct Args {
    /// Source file location
    #[arg(short, long)]
    pub source_path: String,

    /// Output images root
    #[arg(short, long)]
    pub output_root: String,

    /// ID fields indexes
    #[arg(short, long, value_delimiter = ',', default_values_t = [0])]
    pub fields: Vec<i8>,

    /// URL field index
    #[arg(short, long, default_value_t = -1)]
    pub url_field: i8,

    /// Timeout for requests, in seconds
    #[arg(short, long, default_value_t = 5)]
    pub timeout: u64,

    /// Output images max size
    #[arg(short, long, default_value_t = 640)]
    pub max_size: u32,

    /// Output images extension
    #[arg(short, long, default_value_t = DEFAULT_EXTENSION.to_string())]
    pub extension: String,

    /// Output images quality
    #[arg(short, long, default_value_t = 92)]
    pub quality: u8,

    /// Concurrent workers count
    #[arg(short, long, default_value_t = num_cpus::get() * 2)]
    pub worker_count: usize,

    /// Resume last run if any
    #[arg(short, long)]
    pub resume: bool,

    /// Log the results
    #[command(flatten)]
    pub verbose: Verbosity<WarnLevel>,

    /// Show progressbar
    #[arg(short, long)]
    pub progress: bool,

    /// Use the first line in source
    #[arg(short, long)]
    pub no_header: bool,
}
