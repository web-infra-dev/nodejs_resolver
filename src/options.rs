use std::{collections::HashSet, hash::Hash, path::PathBuf, sync::Arc};

use hashlink::LinkedHashMap;

use crate::Cache;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum AliasKind {
    Target(String),
    Ignored,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AliasMap(LinkedHashMap<String, AliasKind>);

impl Default for AliasMap {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl AliasMap {
    pub fn get(&self, k: &str) -> Option<&AliasKind> {
        self.0.get(k)
    }

    pub fn insert(&mut self, k: String, v: AliasKind) -> Option<AliasKind> {
        self.0.insert(k, v)
    }

    pub fn contains_key(&self, k: &str) -> bool {
        self.0.contains_key(k)
    }

    pub fn iter(&self) -> hashlink::linked_hash_map::Iter<String, AliasKind> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> hashlink::linked_hash_map::IterMut<String, AliasKind> {
        self.0.iter_mut()
    }
}

impl<'a> IntoIterator for &'a AliasMap {
    type Item = (&'a String, &'a AliasKind);
    type IntoIter = hashlink::linked_hash_map::Iter<'a, String, AliasKind>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut AliasMap {
    type Item = (&'a String, &'a mut AliasKind);
    type IntoIter = hashlink::linked_hash_map::IterMut<'a, String, AliasKind>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl IntoIterator for AliasMap {
    type Item = (String, AliasKind);
    type IntoIter = hashlink::linked_hash_map::IntoIter<String, AliasKind>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<(String, AliasKind)> for AliasMap {
    fn from_iter<T: IntoIterator<Item = (String, AliasKind)>>(iter: T) -> Self {
        Self(LinkedHashMap::from_iter(iter))
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum EnforceExtension {
    Enabled,
    Disabled,
    Auto,
}

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
    pub alias: AliasMap,
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
        let alias = AliasMap::default();
        let symlinks = true;
        let browser_field = false;
        let condition_names: HashSet<String> =
            HashSet::from_iter(["node"].into_iter().map(String::from));
        let prefer_relative = false;
        let enforce_extension = EnforceExtension::Auto;
        let tsconfig = None;
        let external_cache = None;
        Self {
            prefer_relative,
            extensions,
            main_files,
            main_fields,
            description_file,
            alias,
            symlinks,
            browser_field,
            condition_names,
            enforce_extension,
            tsconfig,
            external_cache,
        }
    }
}
