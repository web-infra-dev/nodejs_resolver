use std::sync::Arc;

use crate::{description::PkgFileInfo, kind::PathKind, Resolver, MODULE};

use super::Plugin;
use crate::{
    map::{ExportsField, Field},
    ResolverInfo, ResolverStats,
};

pub struct ExportsFieldPlugin<'a> {
    pkg_info: &'a Option<Arc<PkgFileInfo>>,
}

impl<'a> ExportsFieldPlugin<'a> {
    pub fn new(pkg_info: &'a Option<Arc<PkgFileInfo>>) -> Self {
        Self { pkg_info }
    }

    fn is_in_module(&self, pkg_info: &PkgFileInfo) -> bool {
        pkg_info.abs_dir_path.to_string_lossy().contains(MODULE)
    }

    pub(crate) fn is_resolve_self(pkg_info: &PkgFileInfo, info: &ResolverInfo) -> bool {
        pkg_info
            .name
            .as_ref()
            .map(|pkg_name| info.request.target.starts_with(pkg_name))
            .map_or(false, |ans| ans)
    }
}

impl<'a> Plugin for ExportsFieldPlugin<'a> {
    fn apply(&self, resolver: &Resolver, info: ResolverInfo) -> ResolverStats {
        if let Some(pkg_info) = self.pkg_info.as_ref() {
            let target = &info.request.target;

            if !info.request.kind.eq(&PathKind::Normal) {
                return ResolverStats::Resolving(info);
            }

            if !self.is_in_module(pkg_info) && !Self::is_resolve_self(pkg_info, &info) {
                return ResolverStats::Resolving(info);
            }

            let list = if let Some(root) = &pkg_info.exports_field_tree {
                let query = &info.request.query;
                let fragment = &info.request.fragment;
                let chars: String = if target.starts_with('@') {
                    let index = target.find('/').unwrap();
                    &target[index + 1..]
                } else {
                    target
                }
                .chars()
                .collect();

                let target = match chars.find('/').map(|index| &chars[index..]) {
                    Some(target) => format!(".{target}"),
                    None => {
                        let target = target.to_string();
                        if info.path.join(&target).exists() || pkg_info.name.eq(&Some(target)) {
                            ".".to_string()
                        } else {
                            return ResolverStats::Error((Resolver::raise_tag(), info));
                        }
                    }
                };
                let remaining_target = if !query.is_empty() || !fragment.is_empty() {
                    let target = if target == "." {
                        String::from("./")
                    } else {
                        target
                    };
                    format!("{target}{query}{fragment}")
                } else {
                    target
                };

                match ExportsField::field_process(
                    root,
                    &remaining_target,
                    &resolver.options.condition_names,
                ) {
                    Ok(list) => list,
                    Err(err) => return ResolverStats::Error((err, info)),
                }
            } else {
                return ResolverStats::Resolving(info);
            };

            for item in list {
                let request = resolver.parse(&item);
                let info = ResolverInfo::from(pkg_info.abs_dir_path.to_path_buf(), request);

                if info.get_path().is_file() && ExportsField::check_target(&info.request.target) {
                    return ResolverStats::Resolving(info);
                }
            }
            ResolverStats::Error((format!("Package path {target} is not exported"), info))
            // TODO: `info.abs_dir_path.as_os_str().to_str().unwrap(),` has abs_path
        } else {
            ResolverStats::Resolving(info)
        }
    }
}
