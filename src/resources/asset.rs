use std::path::Path;

use thiserror::Error;

pub trait Asset: Sized {
    fn load(path: &Path) -> Result<Self, AssetError>;
}

#[derive(Debug, Error)]
pub enum AssetError {
    #[error("failed to read asset at {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to decode asset at {path}: {source}")]
    Decode {
        path: String,
        #[source]
        source: image::ImageError,
    },
    #[error("invalid map at {path}: {reason}")]
    InvalidMap { path: String, reason: String },
    #[error("unsupported loader for {kind}: {hint}")]
    UnsupportedLoader { kind: &'static str, hint: &'static str },
    #[error("unknown asset kind: {0}")]
    UnknownKind(String),
}
