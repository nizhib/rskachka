use clap::Parser;

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

    /// Images root
    #[arg(short, long)]
    pub root: String,

    /// URL field index
    #[arg(short, long, default_value_t = -1)]
    pub url_field: i8,

    /// Skip the first line as header
    #[arg(short, long)]
    pub no_header: bool,

    /// Show progressbar
    #[arg(short, long)]
    pub progress: bool,
}
