use crate::info::NormalizedPath;
use crate::map::{ExportsField, Field, ImportsField, PathTreeNode};
use crate::{AliasMap, Error, RResult};
use dashmap::DashMap;
use once_cell::sync::OnceCell;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug)]
pub struct PkgJSON {
    name: Option<Box<str>>,
    alias_fields: OnceCell<Vec<(String, AliasMap)>>,
    exports_field_map: DashMap<String, RResult<Option<PathTreeNode>>>,
    imports_field_tree: OnceCell<RResult<Option<PathTreeNode>>>,
    raw: serde_json::Value,
}

impl PkgJSON {
    pub(crate) fn parse(content: &str, file_path: &Path) -> RResult<Self> {
        let json: serde_json::Value =
            tracing::debug_span!("serde_json_from_str").in_scope(|| {
                serde_json::from_str(content)
                    .map_err(|error| Error::UnexpectedJson((file_path.into(), error)))
            })?;

        let name = json.get("name").and_then(|v| v.as_str()).map(|s| s.into());

        Ok(Self {
            name,
            alias_fields: OnceCell::new(),
            exports_field_map: DashMap::new(),
            imports_field_tree: OnceCell::new(),
            raw: json,
        })
    }

    pub fn alias_fields(&self) -> &Vec<(String, AliasMap)> {
        self.alias_fields.get_or_init(|| {
            let mut alias_fields = Vec::new();

            if let Some(value) = self.raw.get("browser") {
                // https://github.com/defunctzombie/package-browser-field-spec
                if let Some(map) = value.as_object() {
                    for (key, value) in map {
                        if let Some(false) = value.as_bool() {
                            alias_fields.push((key.to_string(), AliasMap::Ignored));
                        } else if let Some(s) = value.as_str() {
                            alias_fields.push((key.to_string(), AliasMap::Target(s.to_string())));
                        }
                    }
                } else if let Some(false) = value.as_bool() {
                    alias_fields.push((String::from("."), AliasMap::Ignored));
                } else if let Some(s) = value.as_str() {
                    alias_fields.push((String::from("."), AliasMap::Target(s.to_string())));
                }
            }
            alias_fields
        })
    }

    fn get_filed(&self, field: &Vec<String>) -> Option<&serde_json::Value> {
        let mut current_value = self.raw();
        for current_field in field {
            if !current_value.is_object() {
                return None;
            }
            match current_value.get(current_field) {
                Some(next) => current_value = next,
                None => return None,
            };
        }
        Some(current_value)
    }

    pub fn exports_tree(
        &self,
        field: &Vec<String>,
    ) -> dashmap::mapref::one::Ref<String, RResult<Option<PathTreeNode>>> {
        let field_key = field.join(">");
        let entry = self.exports_field_map.entry(field_key);
        let entry = if let Some(field_value) = self.get_filed(field) {
            entry.or_insert_with(|| ExportsField::build_field_path_tree(field_value).map(Some))
        } else {
            entry.or_insert_with(|| Ok(None))
        };
        entry.downgrade()
    }

    pub fn imports_tree(&self) -> &RResult<Option<PathTreeNode>> {
        let tree = self.imports_field_tree.get_or_init(|| {
            self.raw
                .get("imports")
                .map(ImportsField::build_field_path_tree)
                .map_or(Ok(None), |v| v.map(Some))
        });
        tree
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn raw(&self) -> &serde_json::Value {
        &self.raw
    }
}

#[derive(Debug, Clone)]
pub struct DescriptionData {
    json: Arc<PkgJSON>,
    /// The path to the directory where the description file located.
    /// It not a property in package.json.
    dir_path: NormalizedPath,
}

impl DescriptionData {
    pub fn new<P: AsRef<Path>>(json: PkgJSON, dir_path: P) -> Self {
        Self {
            json: Arc::new(json),
            dir_path: NormalizedPath::new(dir_path),
        }
    }

    pub fn dir(&self) -> &NormalizedPath {
        &self.dir_path
    }

    pub fn data(&self) -> &PkgJSON {
        &self.json
    }
}
