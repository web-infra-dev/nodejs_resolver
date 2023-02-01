use crate::entry::EntryStat;
use rustc_hash::FxHasher;
use std::path::Path;
use std::sync::Arc;
use std::{fmt::Debug, hash::BuildHasherDefault, path::PathBuf};

use dashmap::DashMap;

use std::time::Duration;

#[derive(Debug, Default)]
pub struct CachedFS {
    entries: DashMap<PathBuf, Arc<FileEntry>, BuildHasherDefault<FxHasher>>,
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
            let mtime = file_stat.modified().unwrap();
            // debounce
            let interval = Duration::from_millis(300);
            if mtime
                .duration_since(cached.stat.modified().unwrap())
                .unwrap()
                < interval
            {
                return Ok(cached.content.clone());
            }
        }

        let content = std::fs::read_to_string(path)?;
        // update
        let value = Arc::new(FileEntry {
            content: content.clone(),
            stat: *file_stat,
        });
        self.entries.insert(path.to_path_buf(), value);
        Ok(content)
    }
}
