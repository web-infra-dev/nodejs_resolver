use crate::{Error, RResult, ResolveResult, Resolver};

use path_absolutize::Absolutize;

use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

pub trait NormalizePath {
    fn normalize(&self) -> Cow<Path>;
}

impl NormalizePath for Path {
    #[inline]
    fn normalize(&self) -> Cow<Path> {
        // perf: this method does not re-allocate memory if the path does not contain any dots.
        self.absolutize_from(Path::new("")).unwrap()
    }
}

impl Resolver {
    #[tracing::instrument]
    fn normalize_path(&self, path: &Path) -> RResult<PathBuf> {
        if self.options.symlinks {
            let entry = self.load_entry(path)?;
            entry.symlink().map_err(Error::Io)
        } else {
            Ok(path.normalize().to_path_buf())
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
