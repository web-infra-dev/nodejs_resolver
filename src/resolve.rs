use std::path::PathBuf;
use std::sync::Arc;

use crate::{
    description::DescriptionFileInfo,
    kind::PathKind,
    map::{ExportsField, Field, ImportsField},
    RResult, ResolveResult, Resolver, ResolverResult, ResolverStats, Stats,
};

impl Resolver {
    pub(crate) fn resolve_as_file(&self, stats: &Stats) -> ResolverResult {
        let path = stats.get_path();
        if !(*self.options.enforce_extension.as_ref().unwrap_or(&false)) && path.is_file() {
            Ok(ResolveResult::Path(path))
        } else {
            for extension in &self.options.extensions {
                let str = if extension.is_empty() { "" } else { "." };
                let path = if stats.request.target.is_empty() {
                    PathBuf::from(&format!("{}{str}{extension}", stats.dir.display()))
                } else {
                    stats
                        .dir
                        .join(format!("{}{str}{extension}", stats.request.target))
                };
                if path.is_file() {
                    return Ok(ResolveResult::Path(path));
                }
            }

            Err(String::new())
        }
    }

    pub(crate) fn resolve_as_dir(&self, mut stats: Stats, is_in_module: bool) -> ResolverStats {
        let original_dir = stats.dir.clone();
        let dir = original_dir.join(&*stats.request.target);
        if !dir.is_dir() {
            return Resolver::raise(&stats.dir, &stats.request.target);
        }
        let info_wrap = self.load_description_file(&dir)?;
        let is_same_dir = if let Some(info) = &info_wrap {
            dir.eq(&info.abs_dir_path)
        } else {
            false
        };
        stats = stats.with_dir(dir);
        if is_same_dir {
            let info = info_wrap.as_ref().unwrap();
            for main_field in &info.main_fields {
                stats = match self.get_real_target(
                    stats.with_target(main_field.to_string()),
                    &Self::get_target_kind(main_field),
                    &info_wrap,
                    is_in_module,
                )? {
                    Some(stats) => stats,
                    None => return Ok(None),
                };
                // TODO: should be optimized
                let file = self.resolve_as_file(&stats);
                let stats = if file.is_err() && !stats.dir.eq(&original_dir) {
                    self.resolve_as_dir(stats.clone(), is_in_module)
                } else if let Ok(ResolveResult::Path(path)) = file {
                    return Ok(Some(stats.with_dir(path).with_target(String::new())));
                } else {
                    Err("".to_string())
                };
                if stats.is_ok() {
                    return stats;
                }
            }
        }

        for main_file in &self.options.main_files {
            let is_in_module = if let Some(info) = &info_wrap {
                info.abs_dir_path
                    .display()
                    .to_string()
                    .contains("node_modules")
            } else {
                false
            };
            stats = match self.get_real_target(
                stats.with_target(format!("./{main_file}")),
                &PathKind::Relative,
                &info_wrap,
                is_in_module,
            )? {
                Some(stats) => stats,
                None => return Ok(None),
            };
            let file = self.resolve_as_file(&stats);
            if let Ok(ResolveResult::Path(path)) = file {
                return Ok(Some(stats.with_dir(path).with_target(String::new())));
            } else if let Ok(ResolveResult::Ignored) = file {
                return Ok(None);
            }
        }
        Err(String::new())
    }

    pub(crate) fn resolve_as_modules(&self, mut stats: Stats) -> ResolverStats {
        let original_dir = stats.dir.clone();
        for module in &self.options.modules {
            let module_path = original_dir.join(&module);
            if module_path.is_dir() {
                let target = &stats.request.target;
                let info = self.load_description_file(&module_path.join(&**target))?;
                let kind = Self::get_target_kind(target);

                stats =
                    match self.get_real_target(stats.with_dir(module_path), &kind, &info, true)? {
                        Some(stats) => stats,
                        None => return Ok(None),
                    };

                // TODO: should be optimized
                let file = self.resolve_as_file(&stats);
                let result = if file.is_err() && !stats.dir.eq(&original_dir) {
                    self.resolve_as_dir(stats.clone(), true)
                } else if let Ok(ResolveResult::Path(path)) = file {
                    return Ok(Some(stats.with_dir(path).with_target(String::new())));
                } else {
                    return Ok(None);
                };

                if result.is_ok() {
                    return result;
                }
            }

            if let Some(parent_dir) = original_dir.parent() {
                let result =
                    self.resolve_as_modules(stats.clone().with_dir(parent_dir.to_path_buf()));
                if result.is_ok() {
                    return result;
                }
            }
        }
        Resolver::raise(&stats.dir, &stats.request.target)
    }

    fn deal_with_alias_fields_in_info(
        &self,
        target: &Option<String>,
        info: &DescriptionFileInfo,
    ) -> Option<String> {
        target.as_ref().and_then(|target| {
            if info.alias_fields.contains_key(target) {
                info.alias_fields
                    .get(target)
                    .and_then(|next| self.deal_with_alias_fields_in_info(next, info))
            } else {
                Some(target.clone())
            }
        })
    }

    fn deal_with_imports_exports_field_in_info(
        &self,
        stats: Stats,
        info: &DescriptionFileInfo,
    ) -> RResult<Option<Stats>> {
        let target = &stats.request.target;
        let is_imports_field = target.starts_with('#');

        let list = if is_imports_field {
            if let Some(root) = &info.imports_field_tree {
                ImportsField::field_process(root, target, &self.options.condition_names)?
            } else {
                return Ok(Some(stats));
            }
        } else if let Some(root) = &info.exports_field_tree {
            let query = &stats.request.query;
            let fragment = &stats.request.fragment;
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
            return Ok(Some(stats));
        };

        for item in list {
            let request = Self::parse(&item);
            let kind = Resolver::get_target_kind(&request.target);
            let is_normal_kind = matches!(kind, PathKind::Normal);
            let stats = Stats::from(
                if is_imports_field && is_normal_kind {
                    // TODO: check more and use `modules`
                    info.abs_dir_path.join("node_modules")
                } else {
                    info.abs_dir_path.to_path_buf()
                },
                request,
            );

            let path = stats.dir.join(&*stats.request.target);
            if is_imports_field {
                return Ok(Some(if is_normal_kind {
                    let info = self.load_description_file(&path)?;
                    if let Some(info) = &info {
                        if !info
                            .abs_dir_path
                            .display()
                            .to_string()
                            .contains("node_modules")
                        {
                            return Ok(Some(stats));
                        }
                    }
                    match self.get_real_target(stats, &kind, &info, true)? {
                        Some(stats) => stats,
                        None => return Ok(None),
                    }
                } else {
                    stats
                }));
            } else if path.is_file() && ExportsField::check_target(&stats.request.target) {
                return Ok(Some(stats));
            }
        }
        Err(format!("Package path {target} is not exported",))
        // TODO: `info.abs_dir_path.as_os_str().to_str().unwrap(),` has abs_path
    }

    pub(crate) fn get_real_target(
        &self,
        stats: Stats,
        kind: &PathKind,
        description_file_info: &Option<Arc<DescriptionFileInfo>>,
        is_in_module: bool,
    ) -> RResult<Option<Stats>> {
        Ok(if let Some(info) = description_file_info {
            let description_file_dir = &info.abs_dir_path;
            // Should deal `exports` and `imports` firstly.
            // TODO: should optimized
            let stats = if is_in_module {
                match self.deal_with_imports_exports_field_in_info(stats, info)? {
                    Some(stats) => stats,
                    None => return Ok(None),
                }
            } else {
                stats
            };

            let target = &stats.request.target;
            let path = stats.dir.join(&**target);
            // Then `alias_fields`
            for (relative_path, converted_target) in &info.alias_fields {
                if matches!(kind, PathKind::Normal | PathKind::Internal) && target.eq(relative_path)
                {
                    return Ok(self
                        .deal_with_alias_fields_in_info(converted_target, info)
                        .map(|converted| {
                            stats
                                .with_dir(description_file_dir.to_path_buf())
                                .with_target(converted)
                        }));
                }

                let should_converted_path = description_file_dir.join(relative_path);

                if should_converted_path.eq(&path)
                    || self
                        .options
                        .extensions
                        .iter()
                        .any(|ext| should_converted_path.eq(&path.with_extension(ext)))
                {
                    return Ok(self
                        .deal_with_alias_fields_in_info(converted_target, info)
                        .map(|converted| {
                            stats
                                .with_dir(description_file_dir.to_path_buf())
                                .with_target(converted)
                        }));
                }
                // TODO: when trigger main filed
            }
            Some(stats)
        } else {
            Some(stats)
        })
    }
}
