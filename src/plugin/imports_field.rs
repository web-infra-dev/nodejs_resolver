use super::Plugin;
use crate::{
    description::PkgInfo,
    map::{Field, ImportsField},
    PathKind, ResolveInfo, Resolver, ResolverError, ResolverStats, MODULE,
};

pub struct ImportsFieldPlugin<'a> {
    pkg_info: &'a PkgInfo,
}

impl<'a> ImportsFieldPlugin<'a> {
    pub fn new(pkg_info: &'a PkgInfo) -> Self {
        Self { pkg_info }
    }

    fn check_target(info: ResolveInfo, target: &str) -> ResolverStats {
        if info.get_path().is_file() && ImportsField::check_target(&info.request.target) {
            ResolverStats::Resolving(info)
        } else {
            ResolverStats::Error((
                ResolverError::UnexpectedValue(format!("Package path {target} is not exported")),
                info,
            ))
        }
    }
}

impl<'a> Plugin for ImportsFieldPlugin<'a> {
    fn apply(&self, resolver: &Resolver, info: ResolveInfo) -> ResolverStats {
        if !info.request.target.starts_with('#') {
            return ResolverStats::Resolving(info);
        }

        let target = &info.request.target;
        let list = if let Some(root) = &self.pkg_info.json.imports_field_tree {
            match ImportsField::field_process(root, target, &resolver.options.condition_names) {
                Ok(list) => list,
                Err(err) => return ResolverStats::Error((err, info)),
            }
        } else {
            return ResolverStats::Resolving(info);
        };

        if let Some(item) = list.first() {
            let request = resolver.parse(item);
            let is_normal_kind = matches!(request.kind, PathKind::Normal);
            let is_internal_kind = matches!(request.kind, PathKind::Internal);
            let info = ResolveInfo::from(
                if is_normal_kind {
                    self.pkg_info.dir_path.join(MODULE)
                } else {
                    self.pkg_info.dir_path.to_path_buf()
                },
                request,
            );

            if is_normal_kind {
                let path = info.get_path();
                // TODO: should optimized
                let pkg_info = match resolver.load_entry(&path) {
                    Ok(entry) => entry.pkg_info.clone(),
                    Err(err) => return ResolverStats::Error((err, info)),
                };
                if let Some(ref pkg_info) = pkg_info {
                    if !pkg_info.dir_path.display().to_string().contains(MODULE) {
                        return ResolverStats::Resolving(info);
                    }
                }

                let stats = resolver._resolve(info.clone());
                if stats.is_success() {
                    stats
                } else {
                    ResolverStats::Resolving(info)
                }
            } else if is_internal_kind {
                self.apply(resolver, info)
            } else {
                ImportsFieldPlugin::check_target(info, target)
            }
        } else {
            ResolverStats::Error((
                ResolverError::UnexpectedValue(format!("Package path {target} is not exported")),
                info,
            ))
        }
    }
}
