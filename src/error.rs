use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Mismatched interpolation arguments")]
    MismatchedArguments,

    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Missing key: {0}")]
    MissingKey(String),

    #[error("Base64 error: {0}")]
    Base64(#[from] base64::DecodeError),
}
