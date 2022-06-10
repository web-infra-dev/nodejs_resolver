use std::path::{Component, Path, PathBuf};

use crate::{RResult, Resolver, ResolverResult};

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

    pub fn normalize_path(&self, path: &Path) -> RResult<PathBuf> {
        if self.options.symlinks {
            Path::canonicalize(path)
                .map_err(|_| "Path normalized failed".to_string())
                .map(|result| PathBuf::from(Self::adjust(result)))
        } else {
            Ok(path
                .components()
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
                }))
        }
    }

    pub(super) fn normalize_result(&self, result: ResolverResult) -> RResult<ResolverResult> {
        match result {
            ResolverResult::Info(info) => {
                assert!(info.request.target.is_empty());
                let result = self.normalize_path(&info.path)?;
                Ok(ResolverResult::Info(info.with_path(result)))
            }
            ResolverResult::Ignored => Ok(ResolverResult::Ignored),
        }
    }
}
