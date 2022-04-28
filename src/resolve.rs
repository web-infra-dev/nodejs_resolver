use std::path::{Path, PathBuf};

use crate::{
    description::DescriptionFileInfo,
    kind::PathKind,
    map::{ExportsField, Field, ImportsField},
    RResult, Resolver, ResolverResult,
};

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
        original_dir: &Path,
        target: &str,
        query: &str,
        fragment: &str,
        is_in_module: bool,
    ) -> ResolverResult {
        let dir = original_dir.join(target);
        if !dir.is_dir() {
            Err("Not found directory".to_string())
        } else {
            // TODO: cache
            let info_wrap = self.load_description_file(&dir)?;
            if let Some(info) = &info_wrap {
                if dir.eq(&info.abs_dir_path) {
                    for main_field in &info.main_fields {
                        let (base_dir, target) = match self.get_real_target(
                            &dir,
                            main_field,
                            query,
                            fragment,
                            &Self::get_path_kind(main_field),
                            &info_wrap,
                            is_in_module,
                        )? {
                            Some((dir, target)) => (dir, target),
                            None => return Ok(None),
                        };

                        // TODO: should be optimized
                        let file = self.resolve_as_file(&base_dir, &target);
                        let result = if file.is_err() && !base_dir.eq(&original_dir) {
                            self.resolve_as_dir(&base_dir, &target, query, fragment, is_in_module)
                        } else {
                            file
                        };
                        if result.is_ok() {
                            return result;
                        }
                    }
                }
            }

            for main_file in &self.options.main_files {
                let main_file = format!("./{main_file}");
                let is_in_module = if let Some(info) = &info_wrap {
                    info.abs_dir_path
                        .as_os_str()
                        .to_str()
                        .unwrap()
                        .contains("node_modules")
                } else {
                    false
                };
                let (base_dir, target) = match self.get_real_target(
                    &dir,
                    &main_file,
                    query,
                    fragment,
                    &PathKind::Relative,
                    &info_wrap,
                    is_in_module,
                )? {
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

    pub(crate) fn resolve_as_modules(
        &self,
        dir: &Path,
        target: &str,
        query: &str,
        fragment: &str,
    ) -> ResolverResult {
        for module in &self.options.modules {
            let module_path = dir.join(module);
            if module_path.is_dir() {
                // TODO: cache
                let info = self.load_description_file(&module_path.join(target))?;
                let (base_dir, target) = match self.get_real_target(
                    &module_path,
                    target,
                    query,
                    fragment,
                    &Self::get_path_kind(target),
                    &info,
                    true,
                )? {
                    Some((dir, target)) => (dir, target),
                    None => return Ok(None),
                };

                // TODO: should be optimized
                let result = self
                    .resolve_as_file(&base_dir, &target)
                    .or_else(|_| self.resolve_as_dir(&base_dir, &target, query, fragment, true));

                if result.is_ok() {
                    return result;
                }
            }
            if let Some(parent_dir) = dir.parent() {
                let result = self.resolve_as_modules(parent_dir, target, query, fragment);
                if result.is_ok() {
                    return result;
                }
            }
        }
        Err("Not found in modules".to_string())
    }

    fn deal_with_alias_fields_in_info(
        &self,
        target: &Option<String>,
        info: &DescriptionFileInfo,
    ) -> Option<String> {
        if let Some(target) = target {
            if info.alias_fields.contains_key(target) {
                info.alias_fields
                    .get(target)
                    .and_then(|next| self.deal_with_alias_fields_in_info(next, info))
            } else {
                Some(target.clone())
            }
        } else {
            None
        }
    }

    fn deal_with_import_export_fields_in_info(
        &self,
        base_dir: &Path,
        target: &str,
        query: &str,
        fragment: &str,
        info: &DescriptionFileInfo,
    ) -> RResult<Option<(PathBuf, String)>> {
        let is_imports_field = target.starts_with('#');

        let list = if is_imports_field {
            if let Some(root) = &info.imports_field_tree {
                ImportsField::field_process(root, target, &self.options.condition_names)?
            } else {
                return Ok(Some((base_dir.to_path_buf(), target.to_string())));
            }
        } else if let Some(root) = &info.exports_field_tree {
            let chars: String = if target.starts_with('@') {
                let index = target.find('/').unwrap();
                &target[index + 1..]
            } else {
                target
            }
            .chars()
            .collect();

            let request = match chars.find('/').map(|index| &chars[index..]) {
                Some(target) => format!(".{target}"),
                None => ".".to_string(),
            };
            let remaining_request = if !query.is_empty() || !fragment.is_empty() {
                let request = if request == "." {
                    "./".to_string()
                } else {
                    request
                };
                format!("{request}{query}{fragment}")
            } else {
                request
            };
            ExportsField::field_process(root, &remaining_request, &self.options.condition_names)?
        } else {
            return Ok(Some((base_dir.to_path_buf(), target.to_string())));
        };
        dbg!(&list);

        for item in list {
            let target = Self::parse(&item).request;
            let kind = Resolver::get_path_kind(&target);
            let is_normal_kind = matches!(kind, PathKind::Normal);
            let dir = if is_imports_field && is_normal_kind {
                // TODO: check more and use `modules`
                info.abs_dir_path.join("node_modules")
            } else {
                info.abs_dir_path.to_path_buf()
            };
            let target_path = dir.join(&target);
            if is_imports_field {
                if is_normal_kind {
                    // TODO: cache
                    let info = self.load_description_file(&target_path)?;
                    if let Some(info) = &info {
                        if !info
                            .abs_dir_path
                            .as_os_str()
                            .to_str()
                            .unwrap()
                            .contains("node_modules")
                        {
                            return Ok(Some((dir, target)));
                        }
                    }
                    let (base_dir, target) = match self
                        .get_real_target(&dir, &target, query, fragment, &kind, &info, true)?
                    {
                        Some((dir, target)) => (dir, target),
                        None => return Ok(None),
                    };
                    return Ok(Some((base_dir, target)));
                } else {
                    return Ok(Some((info.abs_dir_path.to_path_buf(), target)));
                }
            } else if target_path.is_file()
                && Path::canonicalize(&target_path)
                    .unwrap()
                    .starts_with(&info.abs_dir_path)
            {
                // TODO: use https://github.com/webpack/enhanced-resolve/blob/main/lib/util/path.js#L195

                return Ok(Some((info.abs_dir_path.to_path_buf(), target)));
            }
        }
        Err(format!("Package path {target} is not exported",))
        // TODO: `info.abs_dir_path.as_os_str().to_str().unwrap(),` has abs_path
    }

    fn _get_real_target(
        &self,
        base_dir: &Path,
        target: &str,
        query: &str,
        fragment: &str,
        target_kind: &PathKind,
        description_file_info: Option<&DescriptionFileInfo>,
        is_in_module: bool,
    ) -> RResult<Option<(PathBuf, Option<String>)>> {
        let result = if let Some(info) = description_file_info {
            let description_file_dir = &info.abs_dir_path;
            // Should deal `exports` and `imports` firstly.
            // TODO: should optimized
            let (base_dir, target) = if is_in_module {
                match self.deal_with_import_export_fields_in_info(
                    base_dir, target, query, fragment, info,
                )? {
                    Some((dir, target)) => (dir, target),
                    None => return Ok(None),
                }
            } else {
                (base_dir.to_path_buf(), target.to_string())
            };

            let path = base_dir.join(&target);
            // Then `alias_fields`
            for (relative_path, converted_target) in &info.alias_fields {
                if matches!(target_kind, PathKind::Normal | PathKind::Internal)
                    && target.eq(relative_path)
                {
                    return Ok(Some((
                        description_file_dir.clone(),
                        self.deal_with_alias_fields_in_info(converted_target, info),
                    )));
                }

                let should_converted_path = description_file_dir.join(relative_path);

                if should_converted_path.eq(&path)
                    || self
                        .options
                        .extensions
                        .iter()
                        .any(|ext| should_converted_path.eq(&path.with_extension(ext)))
                {
                    return Ok(Some((
                        description_file_dir.clone(),
                        self.deal_with_alias_fields_in_info(converted_target, info),
                    )));
                }
                // TODO: when trigger main filed
            }
            Some((base_dir, Some(target)))
        } else {
            None
        };
        Ok(result)
    }

    /// TODO: change it to part
    pub(crate) fn get_real_target(
        &self,
        dir: &Path,
        request: &str,
        query: &str,
        fragment: &str,
        kind: &PathKind,
        info: &Option<DescriptionFileInfo>,
        is_in_module: bool,
    ) -> RResult<Option<(PathBuf, String)>> {
        Ok(
            match self._get_real_target(
                dir,
                request,
                query,
                fragment,
                kind,
                info.as_ref(),
                is_in_module,
            )? {
                Some((dir, target)) => target.map(|target| (dir, target)),
                None => Some((dir.to_path_buf(), request.to_string())),
            },
        )
    }
}
