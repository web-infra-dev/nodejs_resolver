use std::{collections::HashSet, path::PathBuf, sync::Arc};

use crate::ResolverUnsafeCache;

#[derive(Debug, Clone)]
pub enum AliasMap {
    Target(String),
    Ignored,
}

#[derive(Debug, Clone)]
pub struct ResolverOptions {
    /// Tried detect file with this extension.
    /// Default is `["js", "json", "node"]`
    pub extensions: Vec<String>,
    /// Enforce that a extension from extensions must be used.
    /// Default is `None`
    pub enforce_extension: Option<bool>,
    /// Maps key to value.
    /// `None` means that the value is `false`.
    /// Default is `vec![]`.
    /// The reason for using `Vec` instead `HashMap` to keep the order.
    pub alias: Vec<(String, AliasMap)>,
    /// Prefer to resolve request as relative request and
    /// fallback to resolving as modules.
    /// Default is `false`
    pub prefer_relative: bool,
    /// Cache had stored the processed `description_file` parsing information by default,
    /// but the action is not secure, and when you try to modify a `description_file`,
    /// they will still use the data before the modification.
    /// Default is `false`.
    pub disable_unsafe_cache: bool,
    /// Use of cache defined external, it designed to shared the info of `description_file`
    /// in different.
    ///
    /// - If `disable_unsafe_cache` is `true`, do not use any cache.
    /// - If `disable_unsafe_cache` is `false`,
    /// and `unsafe_cache` is `None`, use default cache in resolver.
    /// - If `disable_unsafe_cache` is `false`,
    /// and `unsafe_cache.is_some()` is true, use this cache.
    ///
    /// Default is `None`.
    pub unsafe_cache: Option<Arc<ResolverUnsafeCache>>,
    /// Whether to resolve the real path when the result
    /// is a symlink.
    /// Default is `true`.
    pub symlinks: bool,
    /// A JSON file to describing this lib information.
    /// Default is `Some("package.json")`.
    pub description_file: Option<String>,
    /// Main file in this directory.
    /// Default is `["index"]`.
    pub main_files: Vec<String>,
    /// Main fields in Description.
    /// Default is `["main"]`.
    pub main_fields: Vec<String>,
    /// The list of alias fields in description files.
    /// Default is `[]`
    pub alias_fields: Vec<String>,
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
}

impl Default for ResolverOptions {
    fn default() -> Self {
        let extensions = vec![
            String::from("js"),
            String::from("json"),
            String::from("node"),
        ];
        let main_files = vec![String::from("index")];
        let main_fields = vec![String::from("main")];
        let description_file = Some(String::from("package.json"));
        let alias = vec![];
        let symlinks = true;
        let alias_fields = vec![];
        let condition_names: HashSet<String> =
            HashSet::from_iter(["node"].into_iter().map(String::from));
        let prefer_relative = false;
        let disable_unsafe_cache = false;
        let enforce_extension = None;
        let tsconfig = None;
        let unsafe_cache = None;
        Self {
            disable_unsafe_cache,
            prefer_relative,
            extensions,
            main_files,
            main_fields,
            description_file,
            alias,
            symlinks,
            alias_fields,
            condition_names,
            enforce_extension,
            tsconfig,
            unsafe_cache,
        }
    }
}
