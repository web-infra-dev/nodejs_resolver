use std::path::{Path, PathBuf};

use crate::{description::DescriptionFileInfo, kind::PathKind, Resolver, ResolverResult};

impl Resolver {
    pub(crate) fn resolve_as_file(&self, base_dir: &Path, target: &str) -> ResolverResult {
        let path = base_dir.join(target);
        if path.is_file() {
            Ok(Some(path))
        } else {
            for extension in &self.options.extensions {
                let path = base_dir.join(format!("{}.{}", target, extension));
                if path.is_file() {
                    return Ok(Some(path));
                }
            }

            Err("Not found file".to_string())
        }
    }

    pub(crate) fn resolve_as_dir(
        &self,
        info: &Option<DescriptionFileInfo>,
        base_dir: &Path,
        target: &str,
    ) -> ResolverResult {
        let dir = base_dir.join(target);
        if !dir.is_dir() {
            Err("Not found directory".to_string())
        } else {
            if let Some(info) = info {
                if dir.eq(&info.abs_dir_path) {
                    for main_field in &info.main_fields {
                        // The base_dir and target must be handled in the `load_description_file` to prevent recursion
                        let result = self.resolve_as_file(&dir, main_field);
                        if result.is_ok() {
                            return result;
                        }
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
        &self,
        base_dir: &Path,
        target: &str,
        info: &Option<DescriptionFileInfo>,
    ) -> ResolverResult {
        for module in &self.options.modules {
            let module_path = base_dir.join(module);
            if module_path.is_dir() {
                let result = self
                    .resolve_as_file(&module_path, target)
                    .or_else(|_| self.resolve_as_dir(info, &module_path, target));
                if result.is_ok() {
                    return result;
                }
            }
            if let Some(parent_dir) = base_dir.parent() {
                let result = self.resolve_as_modules(parent_dir, target, info);
                if result.is_ok() {
                    return result;
                }
            }
        }
        Err("Not found".to_string())
    }

    pub(crate) fn get_real_target(
        &self,
        base_dir: &Path,
        target: &str,
        target_kind: &PathKind,
        description_file_info: &Option<DescriptionFileInfo>,
    ) -> Option<(PathBuf, Option<String>)> {
        if let Some(info) = description_file_info {
            let path = base_dir.join(target);
            let description_file_dir = &info.abs_dir_path;
            for (relative_path, converted_target) in &info.alias_fields {
                if matches!(target_kind, PathKind::NormalModule | PathKind::Internal)
                    && target.eq(relative_path)
                {
                    return Some((description_file_dir.clone(), converted_target.clone()));
                }

                let should_converted_path = description_file_dir.join(relative_path);

                if should_converted_path.eq(&path) {
                    return Some((description_file_dir.clone(), converted_target.clone()));
                }
                for extension in &self.options.extensions {
                    if should_converted_path.eq(&path.with_extension(extension)) {
                        return Some((description_file_dir.clone(), converted_target.clone()));
                    }
                }
            }
            None
        } else {
            None
        }
    }
}
