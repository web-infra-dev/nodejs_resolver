use std::path::Path;

use crate::{Resolver, ResolverResult};

impl Resolver {
    pub(crate) fn resolve_as_file(&self, base_dir: &Path, target: &str) -> ResolverResult {
        let path = base_dir.join(target);
        if path.is_file() {
            Ok(path)
        } else {
            for extension in &self.options.extensions {
                let path = base_dir.join(format!("{}.{}", target, extension));
                if path.is_file() {
                    return Ok(path);
                }
            }

            Err("Not found file".to_string())
        }
    }

    pub(crate) fn resolve_as_dir(&mut self, base_dir: &Path, target: &str) -> ResolverResult {
        let dir = base_dir.join(target);
        if !dir.is_dir() {
            Err("Not found directory".to_string())
        } else {
            if let Some(main_fields) = self.parse_description_file(&dir) {
                for main_field in &main_fields {
                    let result = self.resolve_as_file(&dir, main_field);
                    if result.is_ok() {
                        return result;
                    }
                }
            }
            for main_file in &self.options.main_files {
                let result = self.resolve_as_file(&dir, main_file);
                if result.is_ok() {
                    return result;
                }
            }
            Err("Not found file".to_string())
        }
    }

    pub(crate) fn resolve_as_modules(
        &mut self,
        base_dir: &Path,
        target: &str,
        may_be_dir: bool,
    ) -> ResolverResult {
        let modules = base_dir.join(&self.options.modules);
        if modules.is_dir() {
            let result = if may_be_dir {
                self.resolve_as_dir(&modules, target)
            } else {
                self.resolve_as_file(&modules, target)
            };
            if result.is_ok() {
                return result;
            }
        }
        match base_dir.parent() {
            Some(parent_dir) => self.resolve_as_modules(parent_dir, target, may_be_dir),
            None => Err("Not found".to_string()),
        }
    }
}
