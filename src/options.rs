use std::collections::HashMap;

use crate::Resolver;

pub struct ResolverOptions {
    pub extensions: Vec<String>,
    pub alias: HashMap<String, String>,
    pub(crate) main_files: Vec<String>,
    pub(crate) main_fields: Vec<String>,
    pub(crate) description_file: String,
    pub(crate) modules: String,
    pub(crate) symlinks: bool,
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
        let description_file = String::from("package.json");
        let alias = HashMap::new();
        let modules = String::from("node_modules");
        let symlinks = true;
        Self {
            extensions,
            main_files,
            main_fields,
            description_file,
            alias,
            modules,
            symlinks,
        }
    }
}

type From = str;
type To = str;

impl Resolver {
    pub fn with_extensions(self, extensions: Vec<&str>) -> Self {
        let extensions = extensions.iter().map(|&s| s.into()).collect();
        let options = ResolverOptions {
            extensions,
            ..self.options
        };
        Self { options, ..self }
    }

    pub fn with_alias(self, alias: Vec<(&From, &To)>) -> Self {
        let alias = alias.iter().map(|&(k, v)| (k.into(), v.into())).collect();
        let options = ResolverOptions {
            alias,
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
}
