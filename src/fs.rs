use dashmap::DashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time;

use crate::RResult;

pub struct CacheFile {
    duration: time::Duration,
    cached_file: DashMap<PathBuf, (Arc<fs::File>, time::SystemTime)>,
}

impl CacheFile {
    /// the unit of `duration` is millisecond.
    pub fn new(duration: u64) -> Self {
        CacheFile {
            duration: time::Duration::from_millis(duration),
            cached_file: Default::default(),
        }
    }

    fn get_last_modified_time_from_file<P: AsRef<Path>>(
        path: P,
        file: &fs::File,
    ) -> RResult<time::SystemTime> {
        file.metadata()
            .map_err(|_| format!("Get metadata of {} failed.", path.as_ref().display()))?
            .modified()
            .map_err(|_| format!("Get modified time of {} failed", path.as_ref().display()))
    }

    pub fn open<P: AsRef<Path>>(&self, path: P) -> RResult<Arc<fs::File>> {
        if let Some(value) = self.cached_file.get(path.as_ref()) {
            let last_modified_time =
                Self::get_last_modified_time_from_file(path.as_ref(), &value.0)?;
            let duration = last_modified_time
                .duration_since(value.1)
                .map_err(|_| format!("Compare SystemTime failed in {}", path.as_ref().display()))?;
            if duration <= self.duration {
                return Ok(value.0.clone());
            }
        }
        let file = Arc::new(
            fs::File::open(path.as_ref())
                .map_err(|_| format!("Open {} failed", path.as_ref().display()))?,
        );
        let last_modified_time = Self::get_last_modified_time_from_file(path.as_ref(), &file)?;
        self.cached_file.insert(
            path.as_ref().to_path_buf(),
            (file.clone(), last_modified_time),
        );
        Ok(file)
    }
}
