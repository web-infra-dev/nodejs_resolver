use std::{collections::HashSet, path::PathBuf, sync::Arc};

use crate::Cache;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum AliasMap {
    Target(String),
    Ignored,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum EnforceExtension {
    Enabled,
    Disabled,
    Auto,
}

pub type Alias = Vec<(String, AliasMap)>;

#[derive(Debug, Clone)]
pub struct Options {
    /// Tried detect file with this extension.
    /// Default is `[".js", ".json", ".node"]`
    pub extensions: Vec<String>,
    /// Enforce that a extension from extensions must be used.
    /// Default is `Auto`.
    pub enforce_extension: EnforceExtension,
    /// Maps key to value.
    /// Default is `vec![]`.
    /// The reason for using `Vec` instead `HashMap` to keep the order.
    pub alias: Alias,
    /// Prefer to resolve request as relative request and
    /// fallback to resolving as modules.
    /// Default is `false`
    pub prefer_relative: bool,
    /// Use of cache defined external, it designed to shared the info of `description_file`
    /// in different resolver.
    ///
    /// - If `external_cache` is `None`, use default cache in resolver.
    /// - If `external_cache.is_some()` is true, use this cache.
    ///
    /// Default is `None`.
    pub external_cache: Option<Arc<Cache>>,
    /// Whether to resolve the real path when the result
    /// is a symlink.
    /// Default is `true`.
    pub symlinks: bool,
    /// A JSON file to describing this lib information.
    /// Default is `"package.json"`.
    pub description_file: String,
    /// Resolve to a context instead of a file.
    /// Default is `false`
    pub resolve_to_context: bool,
    /// Main file in this directory.
    /// Default is `["index"]`.
    pub main_files: Vec<String>,
    /// Main fields in Description.
    /// Default is `["main"]`.
    pub main_fields: Vec<String>,
    /// Whether read and parse `"browser"` filed
    /// in package.json.
    /// Default is `false`
    pub browser_field: bool,
    /// Condition names for exports filed. Note that its
    /// type is a `HashSet`, because the priority is
    /// related to the order in which the export field
    /// fields are written.
    /// Default is `Set(["node"])`.
    pub condition_names: HashSet<String>,
    /// When this filed exists, it tries to read `baseURL`
    /// and `paths` in the corresponding tsconfig,
    /// and processes the mappings.
    /// Default is `None`.
    pub tsconfig: Option<PathBuf>,
    /// A list of directories to resolve modules from, can be absolute path or folder name.
    /// Default is `["node_modules"]`
    pub modules: Vec<String>,
    // Same as `alias`, but only used if default resolving fails.
    // Default is `[]`.
    pub fallback: Alias,
}

impl Default for Options {
    fn default() -> Self {
        let extensions = vec![
            String::from(".js"),
            String::from(".json"),
            String::from(".node"),
        ];
        let main_files = vec![String::from("index")];
        let main_fields = vec![String::from("main")];
        let description_file = String::from("package.json");
        let alias = vec![];
        let symlinks = true;
        let browser_field = false;
        let condition_names: HashSet<String> = HashSet::from([String::from("node")]);
        let prefer_relative = false;
        let enforce_extension = EnforceExtension::Auto;
        let tsconfig = None;
        let external_cache = None;
        let resolve_to_context = false;
        let modules = vec![String::from("node_modules")];
        let fallback = vec![];
        Self {
            fallback,
            modules,
            extensions,
            enforce_extension,
            alias,
            prefer_relative,
            external_cache,
            symlinks,
            description_file,
            resolve_to_context,
            main_files,
            main_fields,
            browser_field,
            condition_names,
            tsconfig,
        }
    }
}
