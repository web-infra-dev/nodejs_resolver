use crate::{DirInfo, RResult, Resolver};
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct DescriptionFileInfo {
    pub abs_dir_path: PathBuf,
    pub main_fields: Vec<String>,
    pub alias_fields: HashMap<String, Option<String>>,
}

impl Resolver {
    fn parse_description_file(&self, path: &Path) -> RResult<DescriptionFileInfo> {
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

        Ok(DescriptionFileInfo {
            abs_dir_path: path.parent().unwrap().to_path_buf(),
            main_fields,
            alias_fields,
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
        now_path: &Path,
    ) -> RResult<Option<DescriptionFileInfo>> {
        if !now_path.is_dir() {
            self.load_description_file(now_path.parent().unwrap())
        } else if let Some(dir) = self.cache_dir_info.get(&now_path.to_path_buf()) {
            Ok(self
                .cache_description_file_info
                .get(&dir.description_file_path)
                .map(|info| info.to_owned()))
        } else {
            match Self::find_description_file_dir(now_path, &self.options.description_file) {
                Some(target_dir) => {
                    let description_file_path = target_dir.join(&self.options.description_file);
                    Ok(Some(self.parse_description_file(&description_file_path)?))
                }
                None => Ok(None),
            }
        }
    }

    pub(crate) fn cache_dir_info(&mut self, now_dir: &Path, description_file_dir: &Path) {
        let mut now_dir = now_dir;
        while now_dir.starts_with(description_file_dir) {
            self.cache_dir_info.insert(
                now_dir.to_path_buf(),
                DirInfo {
                    description_file_path: description_file_dir.to_path_buf(),
                },
            );
            now_dir = now_dir.parent().unwrap();
        }
    }

    pub(crate) fn cache_description_file_info(
        &mut self,
        description_file_info: DescriptionFileInfo,
    ) {
        let abs_dir_path = description_file_info.abs_dir_path.clone();
        self.cache_description_file_info
            .insert(abs_dir_path, description_file_info);
    }
}
