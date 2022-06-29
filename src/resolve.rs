use std::path::{Path, PathBuf};

use smol_str::SmolStr;

use crate::plugin::{
    AliasFieldPlugin, ExportsFieldPlugin, ExtensionsPlugin, ImportsFieldPlugin, MainFieldPlugin,
    MainFilePlugin, Plugin,
};
use crate::utils::RAISE_RESOLVE_ERROR_TAG;
use crate::{Resolver, ResolverInfo, ResolverResult, ResolverStats, MODULE};

impl Resolver {
    pub(crate) fn append_ext_for_path(path: &Path, ext: &str) -> PathBuf {
        let str = if ext.is_empty() { "" } else { "." };
        PathBuf::from(&format!("{}{str}{ext}", path.display()))
    }

    #[tracing::instrument]
    pub(crate) fn resolve_as_file(&self, info: ResolverInfo) -> ResolverStats {
        let path = info.get_path();
        if !(*self.options.enforce_extension.as_ref().unwrap_or(&false)) && path.is_file() {
            ResolverStats::Success(ResolverResult::Info(
                info.with_path(path).with_target(self, ""),
            ))
        } else {
            ExtensionsPlugin::default().apply(self, info)
        }
    }

    #[tracing::instrument]
    pub(crate) fn resolve_as_dir(&self, info: ResolverInfo) -> ResolverStats {
        let dir = info.get_path();
        if !dir.is_dir() {
            return ResolverStats::Error((Resolver::raise_tag(), info));
        }
        let pkg_info_wrap = match self.load_pkg_file(&dir) {
            Ok(pkg_info) => pkg_info,
            Err(err) => return ResolverStats::Error((err, info)),
        };

        let info = info.with_path(dir).with_target(self, "");
        MainFieldPlugin::new(&pkg_info_wrap)
            .apply(self, info)
            .and_then(|info| MainFilePlugin::new(&pkg_info_wrap).apply(self, info))
    }

    #[tracing::instrument]
    pub(crate) fn resolve_as_modules(&self, info: ResolverInfo) -> ResolverStats {
        let original_dir = info.path.clone();
        let module_root_path = original_dir.join(MODULE);
        let stats = if module_root_path.is_dir() {
            let target = &info.request.target;
            // join request part
            let has_namespace_scope = target.starts_with('@');
            let slash_index_list: Vec<usize> = target
                .chars()
                .enumerate()
                .filter(|(_, char)| '/'.eq(char))
                .map(|(index, _)| index)
                .collect();
            let last_start_index = if has_namespace_scope {
                slash_index_list.get(1)
            } else {
                slash_index_list.first()
            };
            let module_name =
                last_start_index.map_or(target.clone(), |&index| SmolStr::new(&target[0..index]));
            let module_path = module_root_path.join(&*module_name);

            let module_info = ResolverInfo::from(module_root_path, info.request.clone());
            let pkg_info = match self.load_pkg_file(&module_info.path.join(&**target)) {
                Ok(pkg_info) => pkg_info,
                Err(err) => return ResolverStats::Error((err, info)),
            };
            let is_resolve_self = pkg_info.as_ref().map_or(false, |pkg_info| {
                ExportsFieldPlugin::is_resolve_self(pkg_info, &module_info)
            });
            if !module_path.is_dir() && !is_resolve_self {
                let stats = self.resolve_as_file(module_info);
                if stats.is_success() {
                    stats
                } else {
                    ResolverStats::Resolving(info)
                }
            } else {
                let stats = ExportsFieldPlugin::new(&pkg_info)
                    .apply(self, module_info)
                    .and_then(|info| ImportsFieldPlugin::new(&pkg_info).apply(self, info))
                    .and_then(|info| AliasFieldPlugin::new(&pkg_info).apply(self, info))
                    .and_then(|info| self.resolve_as_file(info))
                    .and_then(|info| self.resolve_as_dir(info));

                match stats {
                    ResolverStats::Error((error, err_info)) => {
                        if error.eq(RAISE_RESOLVE_ERROR_TAG) {
                            ResolverStats::Resolving(info)
                        } else {
                            ResolverStats::Error((error, err_info))
                        }
                    }
                    _ => stats,
                }
            }
        } else {
            ResolverStats::Resolving(info)
        }
        .and_then(|info| {
            if let Some(parent_dir) = original_dir.parent() {
                self.resolve_as_modules(info.with_path(parent_dir.to_path_buf()))
            } else {
                ResolverStats::Resolving(info)
            }
        });

        match stats {
            ResolverStats::Success(success) => ResolverStats::Success(success),
            ResolverStats::Resolving(info) => ResolverStats::Error((Resolver::raise_tag(), info)),
            ResolverStats::Error(err) => ResolverStats::Error(err),
        }
    }
}
