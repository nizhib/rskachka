use clap::Parser;
use clap_verbosity_flag::{Verbosity, WarnLevel};

#[derive(Parser, Debug)]
#[command(about)]
pub struct Args {
    /// Index file path
    #[arg(short, long)]
    pub index_path: String,

    /// Output images root
    #[arg(short, long)]
    pub output_root: String,

    /// ID fields
    #[arg(short, long, value_delimiter = ',', default_values_t = [0])]
    pub fields: Vec<i8>,

    /// URL field
    #[arg(short, long, default_value_t = -1)]
    pub url_field: i8,

    /// Output images max size
    #[arg(short, long, default_value_t = 640)]
    pub max_size: u32,

    /// Output images jpeg quality
    #[arg(short, long, default_value_t = 90)]
    pub jpeg_quality: u8,

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

    /// No header in index
    #[arg(short, long)]
    pub no_header: bool,
}
