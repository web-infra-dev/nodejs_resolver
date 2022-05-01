use std::path::{Component, Path, PathBuf};

use crate::{Resolver, ResolverResult};

impl Resolver {
    pub fn normalize_alias(&self, target: String) -> Option<String> {
        match self
            .options
            .alias
            .iter()
            .find(|&(key, _)| target.starts_with(key))
        {
            Some((from, to)) => to.as_ref().map(|to| target.replacen(from, to, 1)),
            None => Some(target),
        }
    }

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

    pub fn normalize_path(
        &self,
        path: Option<PathBuf>,
        query: &str,
        fragment: &str,
    ) -> ResolverResult {
        if let Some(path) = path {
            if self.options.symlinks {
                Path::canonicalize(&path)
                    .map_err(|_| "Path normalized failed".to_string())
                    .map(|result| {
                        Some(PathBuf::from(format!(
                            "{}{}{}",
                            Self::adjust(result),
                            query,
                            fragment
                        )))
                    })
            } else {
                let result = path
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
                    });
                Ok(Some(PathBuf::from(format!(
                    "{}{}{}",
                    result.to_str().unwrap(),
                    query,
                    fragment
                ))))
            }
        } else {
            Ok(None)
        }
    }
}
