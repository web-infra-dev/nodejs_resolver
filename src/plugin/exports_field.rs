use crate::{description::PkgInfo, kind::PathKind, Resolver, ResolverError, MODULE};

use super::Plugin;
use crate::{
    map::{ExportsField, Field},
    ResolveInfo, ResolverStats,
};

pub struct ExportsFieldPlugin<'a> {
    pkg_info: &'a PkgInfo,
}

impl<'a> ExportsFieldPlugin<'a> {
    pub fn new(pkg_info: &'a PkgInfo) -> Self {
        Self { pkg_info }
    }

    fn is_in_module(&self, pkg_info: &PkgInfo) -> bool {
        pkg_info.dir_path.to_string_lossy().contains(MODULE)
    }

    pub(crate) fn is_resolve_self(pkg_info: &PkgInfo, info: &ResolveInfo) -> bool {
        pkg_info
            .json
            .name
            .as_ref()
            .map(|pkg_name| info.request.target.starts_with(pkg_name))
            .map_or(false, |ans| ans)
    }
}

impl<'a> Plugin for ExportsFieldPlugin<'a> {
    fn apply(&self, resolver: &Resolver, info: ResolveInfo) -> ResolverStats {
        let target = &info.request.target;

        if !info.request.kind.eq(&PathKind::Normal) {
            return ResolverStats::Resolving(info);
        }

        if !self.is_in_module(self.pkg_info) && !Self::is_resolve_self(self.pkg_info, &info) {
            return ResolverStats::Resolving(info);
        }

        let list = if let Some(root) = &self.pkg_info.json.exports_field_tree {
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
                    if info.path.join(&target).exists() || self.pkg_info.json.name.eq(&Some(target))
                    {
                        ".".to_string()
                    } else {
                        return ResolverStats::Error((ResolverError::ResolveFailedTag, info));
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
            let info = ResolveInfo::from(self.pkg_info.dir_path.to_path_buf(), request);
            if info.get_path().is_file() && ExportsField::check_target(&info.request.target) {
                let stats = resolver._resolve(info);
                if stats.is_success() {
                    return stats;
                }
            }
        }

        ResolverStats::Error((
            ResolverError::UnexpectedValue(format!("Package path {target} is not exported")),
            info,
        ))
        // TODO: `info.abs_dir_path.as_os_str().to_str().unwrap(),` has abs_path
    }
}
