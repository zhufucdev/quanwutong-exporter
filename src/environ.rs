use std::{ffi::OsStr, io, path::Path};

use thiserror::Error;

pub async fn get_env_var(key: impl AsRef<OsStr>) -> Result<String, Error> {
    let value = std::env::var(key)?;
    if value.starts_with("file:") {
        let path = Path::new(value.trim_start_matches("file:"));
        let content = tokio::fs::read(path).await?;
        return Ok(String::from_utf8(content)?);
    }
    Ok(value)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to read file")]
    ReadFile(#[from] io::Error),
    #[error("failed to decode file")]
    Codec(#[from] std::string::FromUtf8Error),
    #[error("{0}")]
    VarError(#[from] std::env::VarError),
}
