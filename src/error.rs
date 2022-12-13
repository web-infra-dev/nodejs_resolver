use std::{io, path::PathBuf};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    UnexpectedJson((PathBuf, serde_json::Error)),
    UnexpectedValue(String),
    ResolveFailedTag,
    Overflow,
    CantFindTsConfig,
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
