use std::path::PathBuf;
use std::sync::Arc;

use super::Plugin;
use crate::{
    description::PkgFileInfo, AliasMap, PathKind, ResolveInfo, ResolveResult, Resolver,
    ResolverStats,
};

pub struct AliasFieldPlugin<'a> {
    pkg_info: &'a Option<Arc<PkgFileInfo>>,
}

impl<'a> AliasFieldPlugin<'a> {
    pub fn new(pkg_info: &'a Option<Arc<PkgFileInfo>>) -> Self {
        Self { pkg_info }
    }

    pub(super) fn request_target_is_module_and_equal_alias_key(
        alias_key: &String,
        info: &ResolveInfo,
    ) -> bool {
        info.request.target.eq(alias_key)
    }

    pub(super) fn request_path_is_equal_alias_key_path(
        alias_path: &PathBuf,
        info: &ResolveInfo,
        extensions: &[String],
    ) -> bool {
        let request_path = info.get_path();
        alias_path.eq(&request_path)
            || extensions.iter().any(|ext| {
                let path_with_extension = Resolver::append_ext_for_path(&request_path, ext);
                alias_path.eq(&path_with_extension)
            })
    }
}

impl<'a> Plugin for AliasFieldPlugin<'a> {
    fn apply(&self, resolver: &Resolver, info: ResolveInfo) -> ResolverStats {
        if !resolver.options.browser_field {
            return ResolverStats::Resolving(info);
        }
        if let Some(pkg_info) = self.pkg_info.as_ref() {
            for (alias_key, alias_target) in &pkg_info.alias_fields {
                let should_deal_alias = match matches!(info.request.kind, PathKind::Normal) {
                    true => Self::request_target_is_module_and_equal_alias_key(alias_key, &info),
                    false => Self::request_path_is_equal_alias_key_path(
                        &pkg_info.abs_dir_path.join(alias_key),
                        &info,
                        &resolver.options.extensions,
                    ),
                };
                if !should_deal_alias {
                    continue;
                }
                match alias_target {
                    AliasMap::Target(converted) => {
                        if alias_key == converted {
                            // pointed itself in `browser` field:
                            // {
                            //  "recursive": "recursive"
                            // }
                            return ResolverStats::Resolving(info);
                        }
                        let alias_info = ResolveInfo::from(
                            pkg_info.abs_dir_path.to_path_buf(),
                            info.request.clone().with_target(converted),
                        );
                        let stats = resolver._resolve(alias_info);
                        if stats.is_success() {
                            return stats;
                        }
                    }
                    AliasMap::Ignored => return ResolverStats::Success(ResolveResult::Ignored),
                };
            }
        }
        ResolverStats::Resolving(info)
    }
}
