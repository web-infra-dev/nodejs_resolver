use crate::{Error, RResult, ResolveResult, Resolver};

use path_absolutize::Absolutize;
use std::path::{Path, PathBuf};

impl Resolver {
    /// Eliminate `\\?\` prefix in windows.
    /// reference: https://stackoverflow.com/questions/41233684/why-does-my-canonicalized-path-get-prefixed-with
    fn adjust(p: PathBuf) -> String {
        const VERBATIM_PREFIX: &str = r#"\\?\"#;
        let p = p.display().to_string();
        if let Some(stripped) = p.strip_prefix(VERBATIM_PREFIX) {
            stripped.to_string()
        } else {
            p
        }
    }

    fn normalize_path_without_link(path: &Path) -> PathBuf {
        // perf: this method does not re-allocate memory if the path does not contain any dots.
        path.absolutize_from(Path::new("")).unwrap().to_path_buf()
    }

    #[tracing::instrument]
    fn normalize_path(&self, path: &Path) -> RResult<PathBuf> {
        let path = Self::normalize_path_without_link(path);
        if self.options.symlinks {
            let entry = self.load_entry(&path)?;
            let symlink = entry.symlink().map_err(Error::Io);
            symlink.map(|result| {
                if cfg!(windows) {
                    PathBuf::from(Self::adjust(result))
                } else {
                    result
                }
            })
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
