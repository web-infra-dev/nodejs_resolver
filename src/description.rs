use crate::map::{ExportsField, Field, ImportsField, PathTreeNode};
use crate::{DirInfo, RResult, Resolver};
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct DescriptionFileInfo {
    pub name: Option<String>,
    pub abs_dir_path: PathBuf,
    pub main_fields: Vec<String>,
    pub alias_fields: HashMap<String, Option<String>>,
    pub exports_field_tree: Option<PathTreeNode>,
    pub imports_field_tree: Option<PathTreeNode>,
}

impl Resolver {
    fn parse_description_file(&self, dir: &Path, target: &str) -> RResult<DescriptionFileInfo> {
        let path = dir.join(target);
        let file = File::open(&path).map_err(|_| "Open failed".to_string())?;

        let json: serde_json::Value = serde_json::from_reader(file)
            .map_err(|_| "Read description file failed".to_string())?;

        let main_fields = self
            .options
            .main_fields
            .iter()
            .fold(vec![], |mut acc, main_filed| {
                if let Some(value) = json.get(main_filed) {
                    // TODO: main_filed maybe a object, array...
                    if let Some(s) = value.as_str() {
                        acc.push(s.to_string());
                    }
                }
                acc
            });

        let mut alias_fields = HashMap::new();
        // TODO: only support ["browser"]
        if self.options.alias_fields.len() == 1 {
            if let Some(value) = json.get(&self.options.alias_fields[0]) {
                if let Some(map) = value.as_object() {
                    for (key, value) in map {
                        // TODO: nested
                        if value.is_boolean() {
                            alias_fields.insert(key.to_string(), None);
                        } else if let Some(s) = value.as_str() {
                            alias_fields.insert(key.to_string(), Some(s.to_string()));
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

        Ok(DescriptionFileInfo {
            name,
            abs_dir_path: dir.to_path_buf(),
            main_fields,
            alias_fields,
            exports_field_tree,
            imports_field_tree,
        })
    }

    fn find_description_file_dir(
        now_dir: &Path,
        description_file_name: &String,
    ) -> Option<PathBuf> {
        let description_path = now_dir.join(description_file_name);
        if description_path.is_file() {
            Some(now_dir.to_path_buf())
        } else {
            now_dir
                .parent()
                .and_then(|parent| Self::find_description_file_dir(parent, description_file_name))
        }
    }

    pub(crate) fn load_description_file(
        &self,
        now_dir: &Path,
    ) -> RResult<Option<DescriptionFileInfo>> {
        if self.options.description_file.is_none() {
            return Ok(None);
        }

        if !now_dir.is_dir() {
            return self.load_description_file(now_dir.parent().unwrap());
        }

        let description_file = if let Some(dir) = self.cache.dir_info.get(&now_dir.to_path_buf()) {
            self.cache
                .description_file_info
                .get(&dir.description_file_path)
                .map(|r#ref| r#ref.clone())
        } else {
            match Self::find_description_file_dir(
                now_dir,
                self.options.description_file.as_ref().unwrap(),
            ) {
                Some(target_dir) => {
                    let parsed = self.parse_description_file(
                        &target_dir,
                        self.options.description_file.as_ref().unwrap(),
                    )?;
                    self.cache.dir_info.insert(
                        now_dir.to_path_buf(),
                        DirInfo {
                            description_file_path: target_dir.to_path_buf(),
                        },
                    );
                    self.cache
                        .description_file_info
                        .insert(target_dir, parsed.clone());
                    Some(parsed)
                }
                None => None,
            }
        };
        Ok(description_file)
    }
}
