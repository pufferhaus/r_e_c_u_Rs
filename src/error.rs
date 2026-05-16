use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("toml parse {file}: {source}")]
    TomlParse {
        file: String,
        #[source]
        source: toml::de::Error,
    },

    #[error("toml serialize {file}: {source}")]
    TomlSerialize {
        file: String,
        #[source]
        source: toml::ser::Error,
    },

    #[error("file not found: {0}")]
    NotFound(PathBuf),

    #[error("gstreamer: {0}")]
    Gst(String),

    #[error("invalid action mapping for key {0}")]
    Keymap(String),

    #[error("other: {0}")]
    Other(String),

    #[error("shader meta parse failed for {file}: {source}")]
    ShaderMeta {
        file: String,
        #[source]
        source: toml::de::Error,
    },

    #[error("shader compile/validate: {0}")]
    ShaderCompile(String),

    #[error("shader not found: {0}")]
    ShaderNotFound(String),
}

pub type Result<T> = std::result::Result<T, Error>;
