use std::{fs, path::PathBuf, sync::atomic::AtomicBool, time::Duration};

use log::info;
use thiserror::Error;

use crate::{
    abort::return_on_flag,
    args::Args,
    fetcher::{FetchError, Fetcher},
    images::{save_bytes_as_image, ImagesError},
    item::{Item, ParsingError},
    saving::SavingSemaphore,
};

pub struct Worker {
    fetcher: Fetcher,
    output_root: String,
    fields: Vec<i8>,
    url_field: i8,
    max_size: u32,
    jpeg_quality: u8,
    resume: bool,
}

impl From<&Args> for Worker {
    fn from(args: &Args) -> Self {
        Worker {
            fetcher: Fetcher::new(Duration::from_secs(args.timeout)),
            output_root: args.output_root.clone(),
            fields: args.fields.clone(),
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

impl Worker {
    pub fn process(
        &self,
        record: &csv::StringRecord,
        stopped: &AtomicBool,
        saving: &SavingSemaphore,
    ) -> Result<(), ProcessError> {
        // Parse the record into an item
        let item = Item::from_record(record, &self.fields, self.url_field)
            .map_err(|e| ProcessError::Custom(format!("Error parsing record: {}", e)))?;
        let path = item_to_path(&item, &self.output_root);

        // Finish if we are resuming and the file exists
        if path.exists() && self.resume {
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
        let bytes = self
            .fetcher
            .fetch(&item.url)
            .map_err(ProcessError::FetchError)?;

        // Process the image and save
        return_on_flag!(stopped, || info!("Shutting down..."));
        save_bytes_as_image(
            &bytes,
            &path,
            self.max_size,
            self.jpeg_quality,
            stopped,
            saving,
        )
        .map_err(ProcessError::ImagesError)
        .map(|_| info!("Saved {}", item.url))
    }
}
