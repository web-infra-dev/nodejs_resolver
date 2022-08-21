use std::path::{Component, Path, PathBuf};

use crate::{RResult, ResolveResult, Resolver, ResolverError};

impl Resolver {
    #[cfg(not(target_os = "windows"))]
    fn adjust(p: PathBuf) -> String {
        p.display().to_string()
    }

    /// Eliminate `\\?\` prefix in windows.
    /// reference: https://stackoverflow.com/questions/41233684/why-does-my-canonicalized-path-get-prefixed-with
    #[cfg(target_os = "windows")]
    fn adjust(p: PathBuf) -> String {
        const VERBATIM_PREFIX: &str = r#"\\?\"#;
        let p = p.display().to_string();
        if p.starts_with(VERBATIM_PREFIX) {
            p[VERBATIM_PREFIX.len()..].to_string()
        } else {
            p
        }
    }

    pub fn normalize_path_without_link(path: &Path) -> PathBuf {
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

    fn normalize_path(&self, path: &Path) -> RResult<PathBuf> {
        if self.options.symlinks {
            Path::canonicalize(path)
                .map_err(ResolverError::Io)
                .map(|result| PathBuf::from(Self::adjust(result)))
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
