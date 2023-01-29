use crate::{Error, RResult, ResolveResult, Resolver};

use path_absolutize::Absolutize;
use std::path::{Path, PathBuf};

impl Resolver {
    fn normalize_path_without_link(path: &Path) -> PathBuf {
        // perf: this method does not re-allocate memory if the path does not contain any dots.
        path.absolutize_from(Path::new("")).unwrap().to_path_buf()
    }

    #[tracing::instrument]
    fn normalize_path(&self, path: &Path) -> RResult<PathBuf> {
        let path = Self::normalize_path_without_link(path);
        if self.options.symlinks {
            let entry = self.load_entry(&path)?;
            entry.symlink().map_err(Error::Io)
        } else {
            Ok(path)
        }
    }

    pub(super) fn normalize_result(&self, result: ResolveResult) -> RResult<ResolveResult> {
        match result {
            ResolveResult::Info(info) => {
                debug_assert!(info.request.target.is_empty());
                let result = self.normalize_path(&info.path)?;
                Ok(ResolveResult::Info(info.with_path(result)))
            }
            ResolveResult::Ignored => Ok(ResolveResult::Ignored),
        }
    }
}
