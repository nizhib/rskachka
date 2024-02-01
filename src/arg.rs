use clap::Parser;
use clap_verbosity_flag::{Verbosity, WarnLevel};

#[derive(Parser, Debug)]
pub struct Arg {
    /// Index file path
    #[arg(short, long)]
    pub file_path: String,

    /// Images output root
    #[arg(short, long)]
    pub output_root: String,

    /// ID fields
    #[arg(short, long, default_value_t = String::from("0"))]
    pub id_fields: String,

    /// URL field
    #[arg(short, long, default_value_t = -1)]
    pub url_field: i8,

    /// Output images quality
    #[arg(short, long, default_value_t = 90)]
    pub jpeg_quality: u8,

    /// Output image size limit
    #[arg(short, long, default_value_t = 640)]
    pub max_size: u32,

    /// Concurrent workers
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

    /// CSV file has no header
    #[arg(short, long)]
    pub no_header: bool,
}
