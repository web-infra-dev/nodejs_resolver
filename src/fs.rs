use std::fmt::Debug;
use std::fs;
use std::path::Path;
use std::time;
use std::time::SystemTime;

use crate::RResult;
use crate::ResolverError;

#[derive(Default, Debug)]
pub struct FileSystem {
    duration: time::Duration,
}

impl FileSystem {
    /// the unit of `duration` is millisecond.
    pub fn new(duration: u64) -> Self {
        Self {
            duration: time::Duration::from_secs(duration),
        }
    }

    fn get_last_modified_time<P: AsRef<Path> + Debug>(path: P) -> RResult<time::SystemTime> {
        fs::metadata(path.as_ref())
            .map_err(ResolverError::Io)?
            .modified()
            .map_err(ResolverError::Io)
    }

    #[tracing::instrument]
    pub fn need_update<P: AsRef<Path> + Debug>(&self, path: P) -> RResult<bool> {
        if !path.as_ref().is_file() {
            // no need update if `p` pointed file dose not exist
            return Ok(false);
        }
        let last_modified_time = Self::get_last_modified_time(path.as_ref())?;
        let now_time = SystemTime::now();
        let duration = now_time.duration_since(last_modified_time).map_err(|_| {
            ResolverError::UnexpectedValue(format!(
                "Compare SystemTime failed in {}",
                path.as_ref().display()
            ))
        })?;
        // if a file changed in `self.duration` seconds,
        // it will reread this file.
        Ok(duration <= self.duration)
    }

    #[tracing::instrument]
    pub fn read_to_string<P: AsRef<Path> + Debug>(&self, path: P) -> RResult<String> {
        fs::read_to_string(path.as_ref()).map_err(ResolverError::Io)
    }
}
