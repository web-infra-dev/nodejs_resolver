use crate::{
    description::DescriptionFileInfo,
    kind::PathKind,
    map::{ExportsField, Field, ImportsField},
    RResult, Resolver, ResolverResult, ResolverStats, Stats,
};

impl Resolver {
    pub(crate) fn resolve_as_file(&self, stats: &Stats) -> ResolverResult {
        let path = stats.get_path();
        if path.is_file() {
            Ok(Some(path))
        } else {
            for extension in &self.options.extensions {
                let path = stats
                    .dir
                    .join(format!("{}.{}", stats.request.target, extension));
                if path.is_file() {
                    return Ok(Some(path));
                }
            }

            Err("Not found file".to_string())
        }
    }

    pub(crate) fn resolve_as_dir(&self, mut stats: Stats, is_in_module: bool) -> ResolverStats {
        let original_dir = stats.dir.to_path_buf();
        let dir = original_dir.join(&stats.request.target);
        if !dir.is_dir() {
            return Err("Not found directory".to_string());
        }
        // TODO: cache
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
                } else if let Ok(Some(path)) = file {
                    return Ok(Some(stats.with_dir(path).with_target(String::new())));
                } else if let Ok(None) = file {
                    return Ok(None);
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
                    .as_os_str()
                    .to_str()
                    .unwrap()
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
            if let Ok(Some(path)) = file {
                return Ok(Some(stats.with_dir(path).with_target(String::new())));
            } else if let Ok(None) = file {
                return Ok(None);
            }
        }
        Err("Not found file".to_string())
    }

    pub(crate) fn resolve_as_modules(&self, mut stats: Stats) -> ResolverStats {
        let original_dir = stats.dir.to_path_buf();
        for module in &self.options.modules {
            let module_path = original_dir.join(&module);
            if module_path.is_dir() {
                let target = &stats.request.target;
                // TODO: cache
                let info = self.load_description_file(&module_path.join(target))?;
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
                } else if let Ok(Some(path)) = file {
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

            let path = stats.dir.join(&stats.request.target);
            if is_imports_field {
                return Ok(Some(if is_normal_kind {
                    // TODO: cache
                    let info = self.load_description_file(&path)?;
                    if let Some(info) = &info {
                        if !info
                            .abs_dir_path
                            .as_os_str()
                            .to_str()
                            .unwrap()
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
        description_file_info: &Option<DescriptionFileInfo>,
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
            let path = stats.dir.join(target);
            // Then `alias_fields`
            for (relative_path, converted_target) in &info.alias_fields {
                if matches!(kind, PathKind::Normal | PathKind::Internal) && target.eq(relative_path)
                {
                    return match self.deal_with_alias_fields_in_info(converted_target, info) {
                        Some(converted) => Ok(Some(
                            stats
                                .with_dir(description_file_dir.to_path_buf())
                                .with_target(converted),
                        )),
                        None => Ok(None),
                    };
                }

                let should_converted_path = description_file_dir.join(relative_path);

                if should_converted_path.eq(&path)
                    || self
                        .options
                        .extensions
                        .iter()
                        .any(|ext| should_converted_path.eq(&path.with_extension(ext)))
                {
                    return match self.deal_with_alias_fields_in_info(converted_target, info) {
                        Some(converted) => Ok(Some(
                            stats
                                .with_dir(description_file_dir.to_path_buf())
                                .with_target(converted),
                        )),
                        None => Ok(None),
                    };
                }
                // TODO: when trigger main filed
            }
            Some(stats)
        } else {
            Some(stats)
        })
    }
}
