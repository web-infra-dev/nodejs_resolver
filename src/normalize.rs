use crate::{RResult, ResolveResult, Resolver, ResolverError};

use std::path::{Component, Path, PathBuf};

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
        path.components()
            .fold(PathBuf::new(), |mut acc, path_component| {
                match path_component {
                    Component::Prefix(prefix) => acc.push(prefix.as_os_str()),
                    Component::Normal(name) => acc.push(name),
                    Component::RootDir => acc.push("/"),
                    Component::CurDir => {}
                    Component::ParentDir => {
                        acc.pop();
                    }
                }
                acc
            })
    }

    #[tracing::instrument]
    fn normalize_path(&self, path: &Path) -> RResult<PathBuf> {
        if self.options.symlinks {
            let entry = self.load_entry(path)?;
            let symlink = entry.symlink().map_err(ResolverError::Io);
            symlink.map(|result| {
                if cfg!(windows) {
                    PathBuf::from(Self::adjust(result))
                } else {
                    result
                }
            })
        } else {
            Ok(Self::normalize_path_without_link(path))
        }
    }

    pub(super) fn normalize_result(&self, result: ResolveResult) -> RResult<ResolveResult> {
        match result {
            ResolveResult::Info(info) => {
                #[cfg(debug_assertions)]
                {
                    assert!(info.request.target.is_empty());
                }

                let result = self.normalize_path(&info.path)?;
                Ok(ResolveResult::Info(info.with_path(result)))
            }
            ResolveResult::Ignored => Ok(ResolveResult::Ignored),
        }
    }
}
