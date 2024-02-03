use std::{fs, path::PathBuf, sync::atomic::AtomicBool};

use log::info;
use thiserror::Error;

use crate::{
    abort::return_on_flag,
    args::Args,
    fetch::{fetch, FetchError},
    images::{save_bytes_as_image, ImagesError},
    item::{Item, ParsingError},
    saving::SavingSemaphore,
};

pub struct ProcessOptions {
    pub output_root: String,
    pub fields: Vec<i8>,
    pub url_field: i8,
    pub max_size: u32,
    pub jpeg_quality: u8,
    pub resume: bool,
}

impl From<Args> for ProcessOptions {
    fn from(args: Args) -> Self {
        ProcessOptions {
            output_root: args.output_root,
            fields: args.fields,
            url_field: args.url_field,
            max_size: args.max_size,
            jpeg_quality: args.jpeg_quality,
            resume: args.resume,
        }
    }
}

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Parsing error: {0}")]
    ParsingError(#[from] ParsingError),

    #[error("Fetch error: {0}")]
    FetchError(#[from] FetchError),

    #[error("URL parsing error: {0}")]
    ImagesError(#[from] ImagesError),

    #[error("Process error: {0}")]
    Custom(String),
}

fn item_to_path(item: &Item, root: &str) -> PathBuf {
    let hash = format!("{:x}", md5::compute(&item.url));
    let name = format!("{}.jpg", &hash[..12]);
    PathBuf::from(root)
        .join(&name[0..2])
        .join(&name[2..4])
        .join(name)
}

pub fn process(
    record: &csv::StringRecord,
    options: &ProcessOptions,
    stopped: &AtomicBool,
    saving: &SavingSemaphore,
) -> Result<(), ProcessError> {
    // Parse the record into an item
    let item = Item::from_record(record, &options.fields, options.url_field)
        .map_err(|e| ProcessError::Custom(format!("Error parsing record: {}", e)))?;
    let path = item_to_path(&item, &options.output_root);

    // Finish if we are resuming and the file exists
    if path.exists() && options.resume {
        info!("Skipping {}", item.url);
        return Ok(());
    }

    // Create all subdirectories
    fs::create_dir_all(path.parent().ok_or_else(|| {
        ProcessError::Custom(format!("Can't infer parent for {}", path.to_str().unwrap()))
    })?)
    .map_err(ProcessError::IO)?;

    // Fetch the record as bytes
    return_on_flag!(stopped, || info!("Shutting down..."));
    let bytes = fetch(&item.url).map_err(ProcessError::FetchError)?;

    // Process the image and save
    return_on_flag!(stopped, || info!("Shutting down..."));
    save_bytes_as_image(
        &bytes,
        &path,
        options.max_size,
        options.jpeg_quality,
        stopped,
        saving,
    )
    .map_err(ProcessError::ImagesError)
    .map(|_| info!("Saved {}", item.url))
}
