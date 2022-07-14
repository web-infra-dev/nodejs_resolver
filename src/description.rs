use crate::map::{ExportsField, Field, ImportsField, PathTreeNode};
use crate::{AliasMap, RResult, Resolver};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum SideEffects {
    Bool(bool),
    Array(Vec<String>),
}

#[derive(Clone, Debug)]
pub struct PkgFileInfo {
    /// The path to the directory where the description file located.
    /// It not a property in package.json.
    pub abs_dir_path: PathBuf,
    pub name: Option<String>,
    pub alias_fields: HashMap<String, AliasMap>,
    pub exports_field_tree: Option<PathTreeNode>,
    pub imports_field_tree: Option<PathTreeNode>,
    pub side_effects: Option<SideEffects>,
    pub raw: serde_json::Value,
}

impl Resolver {
    #[tracing::instrument]
    fn parse_description_file(&self, dir: &Path, file_path: PathBuf) -> RResult<PkgFileInfo> {
        #[cfg(debug_assertions)]
        {
            // ensure that the same package.json is not parsed twice
            if self.dbg_map.contains_key(&file_path) {
                println!("{:?}", self.cache.pkg_info);
                println!("{:?}", self.dbg_map);
                panic!(
                    "Had try to parse same package.json, {}",
                    file_path.display()
                )
            }
            self.dbg_map.insert(file_path.clone(), true);
        }

        let str = tracing::debug_span!("read_to_string").in_scope(|| {
            self.fs
                .read_to_string(&file_path)
                .map_err(|_| format!("Open {} failed", file_path.display()))
        })?;
        let json: serde_json::Value =
            tracing::debug_span!("serde_json_from_str").in_scope(|| {
                serde_json::from_str(&str)
                    .map_err(|_| format!("Parse {} failed", file_path.display()))
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
            Some(ExportsField::build_field_path_tree(value)?)
        } else {
            None
        };

        let imports_field_tree = if let Some(value) = json.get("imports") {
            Some(ImportsField::build_field_path_tree(value)?)
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
                            return Err(format!(
                                "sideEffects in {} had unexpected value {}",
                                file_path.display(),
                                value
                            ));
                        }
                    }
                    Ok(Some(SideEffects::Array(ans)))
                } else {
                    Err(format!(
                        "sideEffects in {} had unexpected value {}",
                        file_path.display(),
                        value
                    ))
                }
            })?;

        Ok(PkgFileInfo {
            name,
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
    pub(crate) fn load_pkg_file(&self, path: &Path) -> RResult<Option<Arc<PkgFileInfo>>> {
        if self.options.description_file.is_none() {
            return Ok(None);
        }
        // Because the key in `self.unsafe_cache.pkg_info` represents directory.
        // So this step is ensure `path` pointed to directory.
        if !path.is_dir() {
            return match path.parent() {
                Some(dir) => self.load_pkg_file(dir),
                None => Err(Resolver::raise_tag()),
            };
        }

        let description_file_name = self.options.description_file.as_ref().unwrap();
        let description_file_path = path.join(description_file_name);
        let need_find_up = if let Some(r#ref) = self.cache.pkg_info.get(path) {
            // if pkg_info_cache contain this key
            if !self.fs.need_update(&description_file_path)? {
                // and not modified, then return
                return Ok(r#ref.clone());
            } else {
                #[cfg(debug_assertions)]
                {
                    self.dbg_map.remove(&description_file_path);
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
                        self.cache.pkg_info.insert(path.to_path_buf(), None);
                        match path.parent() {
                            Some(parent) => path = parent,
                            None => return Ok(None),
                        }
                    }
                }
            }
        }

        let pkg_info = Some(Arc::new(
            self.parse_description_file(path, description_file_path)?,
        ));

        self.cache
            .pkg_info
            .insert(path.to_path_buf(), pkg_info.clone());

        Ok(pkg_info)
    }
}
