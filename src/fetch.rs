use thiserror::Error;

#[derive(Error, Debug)]
pub enum FetchError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Network request error: {0}")]
    Network(Box<ureq::Error>),
}

pub fn fetch(url: &str) -> Result<Vec<u8>, FetchError> {
    let mut buffer = Vec::new();
    ureq::get(url)
        .call()
        .map_err(|e| FetchError::Network(Box::new(e)))?
        .into_reader()
        .read_to_end(&mut buffer)
        .map_err(FetchError::IO)
        .map(|_| buffer)
}
