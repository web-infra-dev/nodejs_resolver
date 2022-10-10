use super::{FileEntry, FileSystem};
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Default)]
pub struct CachedFS {
    entries: DashMap<PathBuf, Arc<FileEntry>>,
}

impl CachedFS {
    pub fn read_file<FS: FileSystem>(&self, fs: &FS, path: &Path) -> std::io::Result<String> {
        let stat = fs.stat(path)?;

        if let Some(cached) = self.entries.get(path) {
            // check cache

            // debounce
            let interval = Duration::from_millis(300);
            if stat
                .modified_time
                .duration_since(cached.stat.modified_time)
                .unwrap()
                > interval
            {
                return Ok(cached.content.clone());
            }
        }

        let content = fs.read_file(path)?;
        Ok(content)
    }
}
