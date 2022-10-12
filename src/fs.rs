use crate::entry::EntryStat;
use std::path::Path;
use std::sync::Arc;
use std::{fmt::Debug, path::PathBuf};

use dashmap::DashMap;

use std::time::Duration;

#[derive(Debug, Default)]
pub struct CachedFS {
    entries: DashMap<PathBuf, Arc<FileEntry>>,
}

#[derive(Debug)]
pub struct FileEntry {
    content: String,
    stat: EntryStat,
}

impl CachedFS {
    pub fn read_file(&self, path: &Path, file_stat: &EntryStat) -> std::io::Result<String> {
        if let Some(cached) = self.entries.get(path) {
            // check cache
            let mtime = file_stat.mtime.as_ref().unwrap();
            // debounce
            let interval = Duration::from_millis(300);
            if mtime.duration_since(cached.stat.mtime.unwrap()).unwrap() < interval {
                return Ok(cached.content.clone());
            }
        }

        let content = std::fs::read_to_string(path)?;
        // update
        let value = Arc::new(FileEntry {
            content: content.clone(),
            stat: file_stat.clone(),
        });
        self.entries.insert(path.to_path_buf(), value);
        Ok(content)
    }
}
