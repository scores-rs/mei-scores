use thiserror::Error;

/// Errors reading or writing MEI.
#[derive(Debug, Error)]
pub enum MeiError {
    #[error("input is not valid utf-8: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("invalid MEI XML: {0}")]
    Xml(String),

    #[error("failed to serialize MEI: {0}")]
    Write(#[from] std::io::Error),
}
