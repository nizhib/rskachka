use thiserror::Error;
use url::Url;

pub struct Item {
    pub id: String,
    pub url: String,
}

#[derive(Error, Debug)]
pub enum ParsingError {
    #[error("URL parsing error: {0}")]
    UrlParse(url::ParseError),

    #[error("Record parsing error: {0}")]
    Custom(String),
}

fn normalize_url(url: &str) -> Result<String, ParsingError> {
    Url::parse(url)
        .map(|parsed| parsed.to_string())
        .map_err(ParsingError::UrlParse)
}

impl Item {
    pub fn from_record(
        record: &csv::StringRecord,
        fields: &[i8],
        url_field: i8,
    ) -> Result<Self, ParsingError> {
        let item_id = fields
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
            .collect::<Result<Vec<&str>, ParsingError>>()?
            .join("$");

        let url_field_idx = if url_field >= 0 {
            url_field as usize
        } else {
            ((url_field + (record.len() as i8)) % (record.len() as i8)) as usize
        };
        let url = record.get(url_field_idx).ok_or_else(|| {
            ParsingError::Custom(format!("Url field index out of bounds: {}", url_field_idx))
        })?;

        Ok(Item {
            id: item_id,
            url: normalize_url(url)?,
        })
    }
}
