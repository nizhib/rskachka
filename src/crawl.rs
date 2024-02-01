use std::{fs, path::PathBuf, sync::Mutex};

use log::info;
use thiserror::Error;
use url::Url;

use crate::{
    abort,
    arg::Arg,
    processing::{self, ProcessingError},
    saving::SavingSemaphore,
};

#[derive(Clone, Debug)]
pub struct CrawlOptions {
    pub output_root: String,
    pub id_fields: String,
    pub url_field: i8,
    pub jpeg_quality: u8,
    pub max_size: u32,
    pub resume: bool,
}

pub struct CrawlItem {
    pub id: String,
    pub url: String,
}

#[derive(Error, Debug)]
pub enum CrawlError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Image processing error: {0}")]
    ImageProcessing(#[from] ProcessingError),

    #[error("URL parsing error for {1}: {0}")]
    UrlParse(url::ParseError, String),

    #[error("Network request error for {1}: {0}")]
    Network(ureq::Error, String),

    #[error("Crawl error: {0}")]
    Custom(String),
}

impl CrawlOptions {
    pub fn from_args(arg: &Arg) -> Self {
        Self {
            output_root: arg.output_root.clone(),
            id_fields: arg.id_fields.clone(),
            url_field: arg.url_field,
            jpeg_quality: arg.jpeg_quality,
            max_size: arg.max_size,
            resume: arg.resume,
        }
    }
}

fn normalize_url(url: &str) -> Result<String, CrawlError> {
    Url::parse(url)
        .map(|parsed| parsed.to_string())
        .map_err(|e| CrawlError::UrlParse(e, url.to_string()))
}

fn record_to_item(
    record: &csv::StringRecord,
    id_fields: &str,
    url_field: i8,
) -> Result<CrawlItem, CrawlError> {
    let id_parts: Vec<&str> = id_fields
        .split(",")
        .map(|field_idx| {
            field_idx
                .parse::<usize>()
                .map_err(|_| CrawlError::Custom(format!("Invalid id field index: {}", field_idx)))
                .and_then(|idx| {
                    record.get(idx).ok_or_else(|| {
                        CrawlError::Custom(format!("Id field index out of bounds: {}", idx))
                    })
                })
        })
        .collect::<Result<Vec<&str>, CrawlError>>()?;

    let item_id = id_parts.join("$");

    let url_field_idx = if url_field >= 0 {
        url_field as usize
    } else {
        ((url_field + (record.len() as i8)) % (record.len() as i8)) as usize
    };
    let url = record.get(url_field_idx).ok_or_else(|| {
        CrawlError::Custom(format!("Url field index out of bounds: {}", url_field_idx))
    })?;

    let image_url = normalize_url(url)?;

    Ok(CrawlItem {
        id: item_id,
        url: image_url,
    })
}

fn url_to_path(url: &str, root: &str) -> PathBuf {
    let hash = format!("{:x}", md5::compute(url));
    let name = format!("{}.jpg", &hash[..12]);
    PathBuf::from(root)
        .join(&name[0..2])
        .join(&name[2..4])
        .join(name)
}

fn fetch(url: &str) -> Result<Vec<u8>, CrawlError> {
    let response = ureq::get(&url)
        .call()
        .map_err(|e| CrawlError::Network(e, url.to_string()))?;
    let mut reader = response.into_reader();
    let mut buffer = Vec::new();
    reader
        .read_to_end(&mut buffer)
        .map_err(|e| CrawlError::IO(e))
        .map(|_| buffer)
}

pub fn get(
    record: &csv::StringRecord,
    options: &CrawlOptions,
    stopped: &Mutex<bool>,
    saving: &SavingSemaphore,
) -> Result<(), CrawlError> {
    // Extract the record fields
    let item = record_to_item(record, &options.id_fields, options.url_field)?;

    // Infer the output filepath
    let path = url_to_path(&item.url, &options.output_root);

    // Finish if we are resuming and the file exists
    if path.exists() && options.resume {
        info!("Skipping previously processed {}", item.url);
        return Ok(());
    }

    // Create all subdirectories
    abort::return_on_flag!(stopped, "Stopping the worker...");
    fs::create_dir_all(path.parent().ok_or_else(|| {
        CrawlError::Custom(format!("Can't find parent for {}", path.to_str().unwrap()))
    })?)
    .map_err(|e| CrawlError::IO(e))?;

    // Fetch the image as bytes
    abort::return_on_flag!(stopped, "Stopping the worker...");
    let bytes = fetch(&item.url)?;

    // Process the image and save
    abort::return_on_flag!(stopped, "Stopping the worker...");
    processing::save_from_bytes(
        &bytes,
        &path,
        options.max_size,
        options.jpeg_quality,
        stopped,
        saving,
    )
    .map_err(|e| CrawlError::ImageProcessing(e))
}
