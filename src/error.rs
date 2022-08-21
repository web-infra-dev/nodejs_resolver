use std::{io, path::PathBuf};

#[derive(Debug)]
pub enum ResolverError {
    Io(io::Error),
    UnexpectedJson((PathBuf, serde_json::Error)),
    UnexpectedValue(String),
    ResolveFailedTag,
}
