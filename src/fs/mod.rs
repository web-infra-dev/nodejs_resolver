mod cached_fs;
use std::fmt::Debug;
use std::path::Path;
use std::time::SystemTime;

use crate::{RResult, Resolver, ResolverError};

pub use cached_fs::*;

#[derive(Debug)]
pub struct Stat {
    modified_time: SystemTime,
}

pub struct DirEntries {}

pub trait FileSystem: Debug {
    fn read_directory(&self, dir: &Path) -> std::io::Result<DirEntries>;
    fn read_file(&self, path: &Path) -> std::io::Result<String>;
    fn stat(&self, path: &Path) -> std::io::Result<Stat>;
}

#[derive(Debug, Default)]
pub struct FS {}

impl FileSystem for FS {
    fn read_directory(&self, _dir: &Path) -> std::io::Result<DirEntries> {
        todo!()
    }

    fn read_file(&self, path: &Path) -> std::io::Result<String> {
        std::fs::read_to_string(path)
    }

    fn stat(&self, path: &Path) -> std::io::Result<Stat> {
        std::fs::metadata(path)
            .and_then(|meta| meta.modified().map(|modified_time| Stat { modified_time }))
    }
}

#[derive(Debug)]
pub struct FileEntry {
    content: String,
    stat: Stat,
}

impl Resolver {
    pub(super) fn read_file(&self, path: &Path) -> RResult<String> {
        self.cache
            .fs
            .read_file(&self.fs, path)
            .map_err(ResolverError::Io)
    }
}
