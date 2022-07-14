use dashmap::DashMap;
use std::fmt::Debug;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time;

use crate::RResult;

#[derive(Default, Debug)]
pub struct CacheFile {
    duration: time::Duration,
    cached_file: DashMap<PathBuf, (Arc<String>, time::SystemTime)>,
}

impl CacheFile {
    /// the unit of `duration` is millisecond.
    pub fn new(duration: u64) -> Self {
        CacheFile {
            duration: time::Duration::from_millis(duration),
            cached_file: Default::default(),
        }
    }

    fn get_last_modified_time_from_file<P: AsRef<Path> + Debug>(
        path: P,
    ) -> RResult<time::SystemTime> {
        fs::metadata(path.as_ref())
            .map_err(|_| format!("Open {} failed", path.as_ref().display()))?
            .modified()
            .map_err(|_| format!("Get modified time of {} failed", path.as_ref().display()))
    }

    #[tracing::instrument]
    pub fn need_update<P: AsRef<Path> + Debug>(&self, path: P) -> RResult<bool> {
        if !path.as_ref().is_file() {
            // no  need update if `p` pointed file dose not exist
            return Ok(false);
        }
        self.cached_file
            .get(path.as_ref())
            .map(|value| value.1)
            .map(|stored_last_modify_time| -> RResult<bool> {
                let duration = Self::get_last_modified_time_from_file(path.as_ref())?
                    .duration_since(stored_last_modify_time)
                    .map_err(|_| {
                        format!("Compare SystemTime failed in {}", path.as_ref().display())
                    })?;
                Ok(duration >= self.duration)
            })
            .map_or(Ok(true), |val| val)
    }

    #[tracing::instrument]
    pub fn read_to_string<P: AsRef<Path> + Debug>(&self, path: P) -> RResult<Arc<String>> {
        let str = Arc::new(
            fs::read_to_string(path.as_ref())
                .map_err(|_| format!("Open {} failed", path.as_ref().display()))?,
        );
        let last_modified_time = Self::get_last_modified_time_from_file(path.as_ref())?;
        self.cached_file.insert(
            path.as_ref().to_path_buf(),
            (str.clone(), last_modified_time),
        );
        Ok(str)
    }
}
