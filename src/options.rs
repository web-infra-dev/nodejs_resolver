use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct ResolverOptions {
    pub extensions: Vec<String>,
    pub alias: HashMap<String, Option<String>>,
    pub condition_names: HashSet<String>,
    pub symlinks: bool,
    pub description_file: Option<String>,
    pub alias_fields: Vec<String>,
    pub main_files: Vec<String>,
    pub main_fields: Vec<String>,
    pub modules: Vec<String>,
    pub prefer_relative: bool,
    pub enable_unsafe_cache: bool,
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
        let alias = HashMap::new();
        let modules = vec![String::from("node_modules")];
        let symlinks = true;
        let alias_fields = vec![];
        let condition_names: HashSet<String> = HashSet::new();
        let prefer_relative = false;
        let enable_unsafe_cache = true;
        Self {
            enable_unsafe_cache,
            prefer_relative,
            extensions,
            main_files,
            main_fields,
            description_file,
            alias,
            modules,
            symlinks,
            alias_fields,
            condition_names,
        }
    }
}
