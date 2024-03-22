use std::path::PathBuf;

use thiserror::Error;
use url::{ParseError, Url};

pub struct Item {
    pub id: String,
    pub url: String,
    pub path: PathBuf,
}

#[derive(Error, Debug)]
pub enum ParsingError {
    #[error("URL parsing error: {0}")]
    UrlParse(url::ParseError),

    #[error("Record parsing error: {0}")]
    Custom(String),
}

fn extract_item_id(record: &csv::StringRecord, fields: &[i8]) -> Result<String, ParsingError> {
    fields
        .iter()
        .map(|&idx| {
            let uidx = if idx >= 0 {
                idx as usize
            } else {
                (idx + (record.len() as i8)) as usize % record.len()
            };
            record.get(uidx).ok_or_else(|| {
                ParsingError::Custom(format!("ID field index out of bounds: {}", uidx))
            })
        })
        .collect::<Result<Vec<&str>, ParsingError>>()
        .map(|v| v.join("$"))
}

fn extract_url(record: &csv::StringRecord, url_field: i8) -> Result<String, ParsingError> {
    let url_field_idx = if url_field >= 0 {
        url_field as usize
    } else {
        ((url_field + (record.len() as i8)) % (record.len() as i8)) as usize
    };
    record
        .get(url_field_idx)
        .ok_or_else(|| {
            ParsingError::Custom(format!("Url field index out of bounds: {}", url_field_idx))
        })
        .map(|s| s.to_string())
}

fn normalize_url(url: &str) -> Result<String, ParseError> {
    Url::parse(url).map(|parsed| parsed.to_string())
}

fn url_to_path(url: &str, root: &str) -> PathBuf {
    let hash = format!("{:x}", md5::compute(url));
    let name = format!("{}.jpg", &hash[..12]);
    PathBuf::from(root)
        .join(&name[0..2])
        .join(&name[0..4])
        .join(name)
}

impl Item {
    pub fn from_record(
        record: &csv::StringRecord,
        fields: &[i8],
        url_field: i8,
        root: &str,
    ) -> Result<Self, ParsingError> {
        let item_id = match fields.len() {
            0 => "n/a".to_string(),
            _ => extract_item_id(record, fields)?,
        };
        let url = extract_url(record, url_field)?;
        let normalized = normalize_url(&url).map_err(ParsingError::UrlParse)?;
        let path = url_to_path(&normalized, root);
        Ok(Item {
            id: item_id,
            url: normalized,
            path,
        })
    }
}
