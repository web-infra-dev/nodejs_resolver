use std::fs;
use std::path::Path;
use std::time::SystemTime;

use crate::{RResult, Resolver, ResolverError};

impl Resolver {
    fn get_last_modified_time(path: &Path) -> RResult<SystemTime> {
        fs::metadata(path)
            .map_err(ResolverError::Io)?
            .modified()
            .map_err(ResolverError::Io)
    }

    #[tracing::instrument]
    pub fn need_update(&self, path: &Path) -> RResult<bool> {
        if !path.is_file() {
            // no need update if `p` pointed file dose not exist
            Ok(false)
        } else if let Some(last_read_time) = self.cache.file_snapshot.get(path) {
            let last_modified_time = Self::get_last_modified_time(path)?;
            if *last_read_time > last_modified_time {
                return Ok(false);
            }
            let now_time = SystemTime::now();
            let duration = now_time.duration_since(last_modified_time).map_err(|_| {
                ResolverError::UnexpectedValue(format!(
                    "Compare SystemTime failed in {}",
                    path.display()
                ))
            })?;
            Ok(duration > self.duration)
        } else {
            Ok(true)
        }
    }

    #[tracing::instrument]
    pub fn read_to_string(&self, path: &Path) -> RResult<String> {
        let now_time = SystemTime::now();
        let content = fs::read_to_string(path).map_err(ResolverError::Io);
        self.cache
            .file_snapshot
            .insert(path.to_path_buf(), now_time);
        content
    }
}
