use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Mismatched interpolation arguments")]
    MismatchedArguments,

    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Missing key: {0}")]
    MissingKey(String),

    #[error("Base64 error: {0}")]
    Base64Error(#[from] base64::DecodeError),
}
