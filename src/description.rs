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
pub struct PkgInfo {
    /// The path to the directory where the description file located.
    /// It not a property in package.json.
    pub abs_dir_path: PathBuf,
    pub name: Option<String>,
    pub version: Option<String>,
    pub alias_fields: HashMap<String, AliasMap>,
    pub exports_field_tree: Option<Arc<PathTreeNode>>,
    pub imports_field_tree: Option<Arc<PathTreeNode>>,
    pub side_effects: Option<SideEffects>,
    pub raw: serde_json::Value,
}

impl Resolver {
    #[tracing::instrument]
    fn parse_description_file(&self, dir: &Path, file_path: &Path) -> RResult<PkgInfo> {
        let str = tracing::debug_span!("read_to_string")
            .in_scope(|| self.fs.read_to_string(&file_path))?;
        let json: serde_json::Value =
            tracing::debug_span!("serde_json_from_str").in_scope(|| {
                serde_json::from_str(&str).map_err(|error| {
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
            let key = serde_json::to_string(&value).unwrap_or_else(|_| {
                panic!("Parse {}/exports to hash key failed", file_path.display())
            });
            if let Some(tree) = self.cache.exports_content_to_tree.get(&key) {
                Some(tree.clone())
            } else {
                let tree = Arc::new(ExportsField::build_field_path_tree(value)?);
                self.cache.exports_content_to_tree.insert(key, tree.clone());
                Some(tree)
            }
        } else {
            None
        };

        let imports_field_tree = if let Some(value) = json.get("imports") {
            let key = serde_json::to_string(&value).unwrap_or_else(|_| {
                panic!("Parse {}/imports to hash key failed", file_path.display())
            });
            if let Some(tree) = self.cache.imports_content_to_tree.get(&key) {
                Some(tree.clone())
            } else {
                let tree = Arc::new(ImportsField::build_field_path_tree(value)?);
                self.cache.exports_content_to_tree.insert(key, tree.clone());
                Some(tree)
            }
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

        Ok(PkgInfo {
            name,
            version,
            abs_dir_path: dir.to_path_buf(),
            alias_fields,
            exports_field_tree,
            imports_field_tree,
            side_effects,
            raw: json,
        })
    }

    pub fn load_side_effects(
        &self,
        path: &Path,
    ) -> RResult<Option<(PathBuf, Option<SideEffects>)>> {
        Ok(self.load_pkg_file(path)?.map(|pkg_info| {
            (
                pkg_info
                    .abs_dir_path
                    .join(self.options.description_file.as_ref().unwrap()),
                pkg_info.side_effects.clone(),
            )
        }))
    }

    #[tracing::instrument]
    pub(crate) fn load_pkg_file(&self, path: &Path) -> RResult<Option<Arc<PkgInfo>>> {
        if self.options.description_file.is_none() {
            return Ok(None);
        }
        // Because the key in `self.cache.file_dir_to_pkg_info` represents directory.
        // So this step is ensure `path` pointed to directory.
        if !path.is_dir() {
            return match path.parent() {
                Some(dir) => self.load_pkg_file(dir),
                None => Err(ResolverError::ResolveFailedTag),
            };
        }

        let description_file_name = self.options.description_file.as_ref().unwrap();
        let description_file_path = path.join(description_file_name);
        let need_find_up = if let Some(r#ref) = self.cache.file_dir_to_pkg_info.get(path) {
            if !self.fs.need_update(&description_file_path)? {
                // and not modified, then return
                return Ok(r#ref.clone());
            } else {
                #[cfg(debug_assertions)]
                {
                    self.cache.debug_read_map.remove(&description_file_path);
                }

                false
            }
        } else {
            !description_file_path.is_file()
        };

        // pkg_info_cache do **not** contain this key
        // or this file had modified
        if need_find_up {
            // find the closest directory witch contains description file
            if let Some(target_dir) = Self::find_up(path, description_file_name) {
                return self.load_pkg_file(&target_dir);
            } else {
                // it means all paths during from the `path` to root pointed None.
                // cache it
                let mut path = path;
                loop {
                    if path.is_dir() {
                        self.cache
                            .file_dir_to_pkg_info
                            .insert(path.to_path_buf(), None);
                        match path.parent() {
                            Some(parent) => path = parent,
                            None => return Ok(None),
                        }
                    }
                }
            }
        }

        // some bugs in multi thread
        // std::thread::sleep(std::time::Duration::from_secs(5));

        #[cfg(debug_assertions)]
        {
            // ensure that the same package.json is not parsed twice
            if self
                .cache
                .debug_read_map
                .contains_key(&description_file_path)
            {
                println!(
                    "Had try to parse parsed package.json, {}",
                    description_file_path.display()
                );
                // println!("{:?}", self.cache.file_dir_to_pkg_info);
                // println!("{:?}", self.cache.debug_read_map);
                // TODO: may panic under multi-thread
                // panic!(
                //     "Had try to parse same package.json, {}",
                //     description_file_path.display()
                // )
            }
        }

        let pkg_info = Some(Arc::new(
            self.parse_description_file(path, &description_file_path)?,
        ));

        self.cache
            .file_dir_to_pkg_info
            .insert(path.to_path_buf(), pkg_info.clone());

        #[cfg(debug_assertions)]
        {
            self.cache
                .debug_read_map
                .insert(&description_file_path, true);
        }

        Ok(pkg_info)
    }
}
