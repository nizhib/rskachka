use clap::Parser;

const DEFAULT_EXTENSION: &str = "webp";

#[derive(Parser, Debug)]
#[command(about)]
pub struct Args {
    /// Source file location
    #[arg(short, long)]
    pub source_path: String,

    /// Index file location
    #[arg(short, long)]
    pub index_path: String,

    /// Missing file location
    #[arg(short, long)]
    pub missing_path: String,

    /// Images output root
    #[arg(short, long)]
    pub output_root: String,

    /// Images extension
    #[arg(short, long, default_value_t = DEFAULT_EXTENSION.to_string())]
    pub extension: String,

    /// URL field index
    #[arg(short, long, default_value_t = -1)]
    pub url_field: i8,

    /// Concurrent workers count
    #[arg(short, long, default_value_t = num_cpus::get() * 2)]
    pub worker_count: usize,

    /// Use the first line in source
    #[arg(short, long)]
    pub no_header: bool,

    /// Show progressbar
    #[arg(short, long)]
    pub progress: bool,
}
