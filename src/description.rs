use crate::map::{ExportsField, Field, ImportsField, PathTreeNode};
use crate::{AliasMap, RResult, Resolver, ResolverError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum SideEffects {
    Bool(bool),
    Array(Vec<String>),
}

#[derive(Clone, Debug)]
pub struct PkgInfoInner {
    pub name: Option<String>,
    pub version: Option<String>,
    // TODO: use `IndexMap`
    pub alias_fields: HashMap<String, AliasMap>,
    pub exports_field_tree: Option<Arc<PathTreeNode>>,
    pub imports_field_tree: Option<Arc<PathTreeNode>>,
    pub side_effects: Option<SideEffects>,
    pub raw: serde_json::Value,
}

impl PkgInfoInner {
    fn parse(content: &str, file_path: &Path) -> RResult<Self> {
        let json: serde_json::Value =
            tracing::debug_span!("serde_json_from_str").in_scope(|| {
                serde_json::from_str(content).map_err(|error| {
                    ResolverError::UnexpectedJson((file_path.to_path_buf(), error))
                })
            })?;

        let mut alias_fields = HashMap::new();

        if let Some(value) = json.get("browser") {
            if let Some(map) = value.as_object() {
                for (key, value) in map {
                    if let Some(b) = value.as_bool() {
                        assert!(!b);
                        alias_fields.insert(key.to_string(), AliasMap::Ignored);
                    } else if let Some(s) = value.as_str() {
                        alias_fields.insert(key.to_string(), AliasMap::Target(s.to_string()));
                    }
                }
            }
        }

        let exports_field_tree = if let Some(value) = json.get("exports") {
            let tree = Arc::new(ExportsField::build_field_path_tree(value)?);
            Some(tree)
        } else {
            None
        };

        let imports_field_tree = if let Some(value) = json.get("imports") {
            let tree = Arc::new(ImportsField::build_field_path_tree(value)?);
            Some(tree)
        } else {
            None
        };

        let name = json
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let side_effects: Option<SideEffects> =
            json.get("sideEffects").map_or(Ok(None), |value| {
                // TODO: should optimized
                if let Some(b) = value.as_bool() {
                    Ok(Some(SideEffects::Bool(b)))
                } else if let Some(vec) = value.as_array() {
                    let mut ans = vec![];
                    for value in vec {
                        if let Some(str) = value.as_str() {
                            ans.push(str.to_string());
                        } else {
                            return Err(ResolverError::UnexpectedValue(format!(
                                "sideEffects in {} had unexpected value {}",
                                file_path.display(),
                                value
                            )));
                        }
                    }
                    Ok(Some(SideEffects::Array(ans)))
                } else {
                    Err(ResolverError::UnexpectedValue(format!(
                        "sideEffects in {} had unexpected value {}",
                        file_path.display(),
                        value
                    )))
                }
            })?;

        let version = json
            .get("version")
            .and_then(|value| value.as_str())
            .map(|str| str.to_string());

        Ok(Self {
            name,
            version,
            alias_fields,
            exports_field_tree,
            imports_field_tree,
            side_effects,
            raw: json,
        })
    }
}

#[derive(Clone, Debug)]
pub struct PkgInfo {
    /// The path to the directory where the description file located.
    /// It not a property in package.json.
    pub abs_dir_path: PathBuf,
    pub inner: Arc<PkgInfoInner>,
}

impl Resolver {
    // #[tracing::instrument]
    // fn parse_description_file(&self, dir: &Path, file_path: &Path) -> RResult<PkgInfo> {
    //     let str = self.read_file(file_path)?;
    //     let inner = PkgInfoInner::parse(&str, file_path);
    //     Ok(PkgInfo {
    //         abs_dir_path: dir.to_path_buf(),
    //         inner: Arc,
    //     })
    // }

    pub fn load_side_effects(
        &self,
        path: &Path,
    ) -> RResult<Option<(PathBuf, Option<SideEffects>)>> {
        Ok(self.load_pkg_file(path)?.map(|pkg_info| {
            (
                pkg_info
                    .abs_dir_path
                    .join(self.options.description_file.as_ref().unwrap()),
                pkg_info.inner.side_effects.clone(),
            )
        }))
    }

    #[tracing::instrument]
    pub(crate) fn load_pkg_file(&self, path: &Path) -> RResult<Option<PkgInfo>> {
        if self.options.description_file.is_none() {
            return Ok(None);
        }
        // Because the key in `self.cache.pkg_info` represents directory.
        // So this step is ensure `path` pointed to directory.
        if !path.is_dir() {
            return match path.parent() {
                Some(dir) => self.load_pkg_file(dir),
                None => Err(ResolverError::ResolveFailedTag),
            };
        }

        let description_file_name = self.options.description_file.as_ref().unwrap();
        let description_file_path = path.join(description_file_name);
        let need_find_up = !description_file_path.is_file();
        // TODO: dir_info_cache
        if need_find_up {
            // find the closest directory witch contains description file
            if let Some(target_dir) = Self::find_up(path, description_file_name) {
                return self.load_pkg_file(&target_dir);
            } else {
                return Ok(None);
            }
        }

        let content = self.read_file(&description_file_path)?;
        let pkg_info = if let Some(inner) = self.cache.pkg_info.get(&content) {
            PkgInfo {
                abs_dir_path: path.to_path_buf(),
                inner: inner.clone(),
            }
        } else {
            let inner = Arc::new(PkgInfoInner::parse(&content, &description_file_path)?);

            self.cache.pkg_info.insert(content, inner.clone());
            PkgInfo {
                abs_dir_path: path.to_path_buf(),
                inner,
            }
        };
        Ok(Some(pkg_info))
    }
}
