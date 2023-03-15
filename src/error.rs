use std::{io, path::Path};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    UnexpectedJson((Box<Path>, serde_json::Error)),
    UnexpectedValue(String),
    ResolveFailedTag,
    Overflow,
    CantFindTsConfig(Box<Path>),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
