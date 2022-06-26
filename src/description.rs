use crate::map::{ExportsField, Field, ImportsField, PathTreeNode};
use crate::{AliasMap, RResult, Resolver};
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct PkgFileInfo {
    /// The path to the directory where the description file located.
    /// It not a property in package.json.
    pub abs_dir_path: PathBuf,
    pub name: Option<String>,
    pub main_fields: Vec<String>,
    pub alias_fields: HashMap<String, AliasMap>,
    pub exports_field_tree: Option<PathTreeNode>,
    pub imports_field_tree: Option<PathTreeNode>,
    pub side_effects: Option<bool>,
}

impl Resolver {
    #[tracing::instrument]
    fn parse_description_file(
        &self,
        dir: &Path,
        description_file_name: &str,
    ) -> RResult<PkgFileInfo> {
        let location = dir.join(description_file_name);
        let file =
            File::open(&location).map_err(|_| format!("Open {} failed", location.display()))?;

        let json: serde_json::Value = serde_json::from_reader(file)
            .map_err(|_| format!("Parse {} failed", location.display()))?;

        let main_fields = self
            .options
            .main_fields
            .iter()
            .fold(vec![], |mut acc, main_filed| {
                if let Some(value) = json.get(main_filed) {
                    // TODO: `main_field` maybe a object, array...
                    if let Some(s) = value.as_str() {
                        acc.push(s.to_string());
                    }
                }
                acc
            });

        let mut alias_fields = HashMap::new();
        for alias_filed in &self.options.alias_fields {
            if let Some(value) = json.get(alias_filed) {
                if let Some(map) = value.as_object() {
                    for (key, value) in map {
                        // TODO: nested
                        if let Some(b) = value.as_bool() {
                            assert!(!b);
                            alias_fields.insert(key.to_string(), AliasMap::Ignored);
                        } else if let Some(s) = value.as_str() {
                            alias_fields.insert(key.to_string(), AliasMap::Target(s.to_string()));
                        }
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

        let side_effects = json.get("sideEffects").and_then(|v| v.as_bool());

        Ok(PkgFileInfo {
            name,
            abs_dir_path: dir.to_path_buf(),
            main_fields,
            alias_fields,
            exports_field_tree,
            imports_field_tree,
            side_effects,
        })
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

        let pkg_info = if let Some(r#ref) = self
            .unsafe_cache
            .as_ref()
            .and_then(|cache| cache.pkg_info.get(path))
        {
            r#ref.clone()
        } else {
            let description_file_name = self.options.description_file.as_ref().unwrap();
            let (pkg_info, target_dir) =
                if let Some(target_dir) = Self::find_up(path, description_file_name) {
                    let parsed =
                        Arc::new(self.parse_description_file(&target_dir, description_file_name)?);
                    (Some(parsed), Some(target_dir))
                } else {
                    (None, None)
                };

            // TODO: should optimized
            if let Some(cache) = self.unsafe_cache.as_ref() {
                let mut temp_dir = path.to_path_buf();
                let target_dir = if let Some(target_dir) = target_dir {
                    target_dir
                } else {
                    PathBuf::from("/")
                };
                loop {
                    let info = pkg_info.clone();
                    cache.pkg_info.insert(temp_dir.clone(), info);
                    if temp_dir.eq(&target_dir) || !temp_dir.pop() {
                        break;
                    }
                }
            }
            pkg_info
        };

        Ok(pkg_info)
    }
}
