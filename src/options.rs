use std::collections::{HashMap, HashSet};

use crate::Resolver;

#[derive(Debug)]
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
        Self {
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

type From = str;
type To = str;

impl Resolver {
    pub fn with_extensions(self, extensions: Vec<&str>) -> Self {
        let extensions = extensions
            .iter()
            .map(|&s| {
                if s.starts_with('.') {
                    s.chars().skip(1).collect()
                } else {
                    s.into()
                }
            })
            .collect();
        let options = ResolverOptions {
            extensions,
            ..self.options
        };
        Self { options, ..self }
    }

    pub fn with_alias(self, alias: Vec<(&From, Option<&To>)>) -> Self {
        let alias = alias
            .iter()
            .map(|&(k, v)| (k.into(), v.map(|v| v.into())))
            .collect();
        let options = ResolverOptions {
            alias,
            ..self.options
        };
        Self { options, ..self }
    }

    pub fn with_alias_fields(self, alias_fields: Vec<&str>) -> Self {
        let alias_fields = alias_fields.iter().map(|&s| s.into()).collect();
        let options = ResolverOptions {
            alias_fields,
            ..self.options
        };
        Self { options, ..self }
    }

    pub fn with_modules(self, modules: Vec<&str>) -> Self {
        let modules = modules.iter().map(|&s| s.into()).collect();
        let options = ResolverOptions {
            modules,
            ..self.options
        };
        Self { options, ..self }
    }

    pub fn with_symlinks(self, symlinks: bool) -> Self {
        let options = ResolverOptions {
            symlinks,
            ..self.options
        };
        Self { options, ..self }
    }

    pub fn with_condition_names(self, condition_names: HashSet<String>) -> Self {
        let options = ResolverOptions {
            condition_names,
            ..self.options
        };
        Self { options, ..self }
    }

    pub fn with_description_file(self, description_file: Option<String>) -> Self {
        let options = ResolverOptions {
            description_file,
            ..self.options
        };
        Self { options, ..self }
    }
}
