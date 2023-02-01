use crate::map::{ExportsField, Field, ImportsField, PathTreeNode};
use crate::{AliasMap, Error, RResult, Resolver};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum SideEffects {
    Bool(bool),
    String(String),
    Array(Vec<String>),
}

#[derive(Debug)]
pub struct PkgJSON {
    pub name: Option<String>,
    pub version: Option<String>,
    pub alias_fields: Vec<(String, AliasMap)>,
    pub exports_field_tree: Option<PathTreeNode>,
    pub imports_field_tree: Option<PathTreeNode>,
    pub side_effects: Option<SideEffects>,
    pub raw: serde_json::Value,
}

#[derive(Debug)]
pub struct PkgInfo {
    pub json: Arc<PkgJSON>,
    /// The path to the directory where the description file located.
    /// It not a property in package.json.
    pub dir_path: Box<Path>,
}

impl PkgJSON {
    pub(crate) fn parse(content: &str, file_path: &Path) -> RResult<Self> {
        let json: serde_json::Value =
            tracing::debug_span!("serde_json_from_str").in_scope(|| {
                serde_json::from_str(content)
                    .map_err(|error| Error::UnexpectedJson((file_path.to_path_buf(), error)))
            })?;

        let mut alias_fields = Vec::new();

        if let Some(value) = json.get("browser") {
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
            } else {
                let msg = format!(
                    "The browser is {} which meet unhandled value, error in {}/package.json",
                    value,
                    file_path.display()
                );
                println!("{msg}");
            }
        }

        let exports_field_tree = if let Some(value) = json.get("exports") {
            let tree = ExportsField::build_field_path_tree(value)?;
            Some(tree)
        } else {
            None
        };

        let imports_field_tree = if let Some(value) = json.get("imports") {
            let tree = ImportsField::build_field_path_tree(value)?;
            Some(tree)
        } else {
            None
        };

        let name = json
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let side_effects: Option<SideEffects> = json.get("sideEffects").and_then(|value| {
            // TODO: should optimized
            if let Some(b) = value.as_bool() {
                Some(SideEffects::Bool(b))
            } else if let Some(s) = value.as_str() {
                Some(SideEffects::String(s.to_owned()))
            } else if let Some(vec) = value.as_array() {
                let mut ans = vec![];
                for value in vec {
                    if let Some(str) = value.as_str() {
                        ans.push(str.to_string());
                    } else {
                        return None;
                    }
                }
                Some(SideEffects::Array(ans))
            } else {
                None
            }
        });

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

impl Resolver {
    pub fn load_side_effects(
        &self,
        path: &Path,
    ) -> RResult<Option<(PathBuf, Option<SideEffects>)>> {
        let entry = self.load_entry(path)?;
        let ans = entry.pkg_info().map(|pkg_info| {
            (
                pkg_info.dir_path.join(&self.options.description_file),
                pkg_info.json.side_effects.clone(),
            )
        });
        Ok(ans)
    }
}
