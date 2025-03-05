use std::string::FromUtf8Error;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("io -- {0}")]
    Io(#[from] std::io::Error),

    #[error("utf8 -- {0}")]
    Utf8(#[from] FromUtf8Error),

    #[error("other -- {0}")]
    Other(#[from] anyhow::Error),
}
