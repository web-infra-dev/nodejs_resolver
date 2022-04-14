use std::path::{Component, Path, PathBuf};

use crate::{Resolver, ResolverResult};

impl Resolver {
    pub fn normalize_alias(&self, target: &str) -> String {
        match self
            .options
            .alias
            .iter()
            .find(|&(key, _)| target.starts_with(key))
        {
            Some((from, to)) => target.replacen(from, to, 1),
            None => target.to_owned(),
        }
    }

    pub fn normalize_path(&self, path: &Path) -> ResolverResult {
        if self.options.symlinks {
            Path::canonicalize(path).map_err(|_| "Path normalized failed".to_string())
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
}
