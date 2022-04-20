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

    pub(crate) fn resolve_as_dir(&self, base_dir: &Path, target: &str) -> ResolverResult {
        let dir = base_dir.join(target);
        if !dir.is_dir() {
            Err("Not found directory".to_string())
        } else {
            // TODO: cache
            let info = self.load_description_file(&dir)?;
            if let Some(info) = &info {
                if dir.eq(&info.abs_dir_path) {
                    for main_field in &info.main_fields {
                        let (base_dir, target) = match self.get_real_target(
                            &dir,
                            main_field,
                            &Self::get_path_kind(main_field),
                            &Some(info.clone()), // TODO: fix clone
                        ) {
                            Some((dir, target)) => (dir, target),
                            None => return Ok(None),
                        };
                        let result = self
                            .resolve_as_file(&base_dir, &target)
                            .or_else(|_| self.resolve_as_dir(&base_dir, &target));
                        if result.is_ok() {
                            return result;
                        }
                    }
                }
            }

            for main_file in &self.options.main_files {
                let (base_dir, target) = match self.get_real_target(
                    &dir,
                    main_file,
                    &Self::get_path_kind(main_file),
                    &info,
                ) {
                    Some((dir, target)) => (dir, target),
                    None => return Ok(None),
                };
                let result = self.resolve_as_file(&base_dir, &target);
                if result.is_ok() {
                    return result;
                }
            }
            Err("Not found file".to_string())
        }
    }

    pub(crate) fn resolve_as_modules(&self, base_dir: &Path, target: &str) -> ResolverResult {
        for module in &self.options.modules {
            let module_path = base_dir.join(module);
            if module_path.is_dir() {
                // TODO: cache
                let info = self.load_description_file(&module_path.join(target))?;
                let (base_dir, target) = match self.get_real_target(
                    &module_path,
                    target,
                    &Self::get_path_kind(target),
                    &info,
                ) {
                    Some((dir, target)) => (dir, target),
                    None => return Ok(None),
                };

                let result = self
                    .resolve_as_file(&base_dir, &target)
                    .or_else(|_| self.resolve_as_dir(&base_dir, &target));
                if result.is_ok() {
                    return result;
                }
            }
            if let Some(parent_dir) = base_dir.parent() {
                let result = self.resolve_as_modules(parent_dir, target);
                if result.is_ok() {
                    return result;
                }
            }
        }
        Err("Not found in modules".to_string())
    }

    fn get_final_convert_in_info(
        converted_target: &Option<String>,
        info: &DescriptionFileInfo,
    ) -> Option<String> {
        if let Some(converted_target) = converted_target {
            if info.alias_fields.contains_key(converted_target) {
                info.alias_fields
                    .get(converted_target)
                    .and_then(|next| Self::get_final_convert_in_info(next, info))
            } else {
                Some(converted_target.clone())
            }
        } else {
            None
        }
    }

    fn _get_real_target(
        &self,
        base_dir: &Path,
        target: &str,
        target_kind: &PathKind,
        description_file_info: Option<&DescriptionFileInfo>,
    ) -> Option<(PathBuf, Option<String>)> {
        description_file_info.and_then(|info| {
            let path = base_dir.join(target);
            let description_file_dir = &info.abs_dir_path;
            for (relative_path, converted_target) in &info.alias_fields {
                if matches!(target_kind, PathKind::Normal | PathKind::Internal)
                    && target.eq(relative_path)
                {
                    return Some((
                        description_file_dir.clone(),
                        Self::get_final_convert_in_info(converted_target, info),
                    ));
                }

                let should_converted_path = description_file_dir.join(relative_path);

                if should_converted_path.eq(&path)
                    || self
                        .options
                        .extensions
                        .iter()
                        .any(|ext| should_converted_path.eq(&path.with_extension(ext)))
                {
                    return Some((
                        description_file_dir.clone(),
                        Self::get_final_convert_in_info(converted_target, info),
                    ));
                }
                // TODO: when trigger main filed
            }
            None
        })
    }

    pub(crate) fn get_real_target(
        &self,
        dir: &Path,
        request: &str,
        kind: &PathKind,
        info: &Option<DescriptionFileInfo>,
    ) -> Option<(PathBuf, String)> {
        match self._get_real_target(&dir, request, &kind, info.as_ref()) {
            Some((dir, target)) => target.map(|target| (dir, target)),
            None => Some((dir.to_path_buf(), request.to_string())),
        }
    }
}
