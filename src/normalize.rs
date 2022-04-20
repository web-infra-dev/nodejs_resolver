use std::path::{Component, Path, PathBuf};

use crate::{parse::Part, Resolver, ResolverResult};

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

    pub fn normalize_path(&self, path: Option<PathBuf>, part: &Part) -> ResolverResult {
        if let Some(path) = path {
            if self.options.symlinks {
                Path::canonicalize(&path)
                    .map_err(|_| "Path normalized failed".to_string())
                    .map(|result| {
                        Some(PathBuf::from(format!(
                            "{}{}{}",
                            result.to_str().unwrap(),
                            part.query,
                            part.fragment
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
                    part.query,
                    part.fragment
                ))))
            }
        } else {
            Ok(None)
        }
    }
}
