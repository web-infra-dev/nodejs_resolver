use crate::{description::PkgFileInfo, AliasMap, Resolver};
use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};

use super::Plugin;
use crate::{PathKind, ResolverInfo, ResolverResult, ResolverStats};

pub struct AliasFieldPlugin<'a> {
    pkg_info: &'a Option<Arc<PkgFileInfo>>,
}

impl<'a> AliasFieldPlugin<'a> {
    pub fn new(pkg_info: &'a Option<Arc<PkgFileInfo>>) -> Self {
        Self { pkg_info }
    }

    pub(super) fn deal_with_alias_fields_recursive(
        &self,
        target: &AliasMap,
        alias_fields: &HashMap<String, AliasMap>,
    ) -> AliasMap {
        match target {
            AliasMap::Target(target) => {
                if let Some(next) = alias_fields.get(target) {
                    self.deal_with_alias_fields_recursive(next, alias_fields)
                } else {
                    AliasMap::Target(target.clone())
                }
            }
            AliasMap::Ignored => AliasMap::Ignored,
        }
    }

    pub(super) fn request_target_is_module_and_equal_alias_key(
        alias_key: &String,
        info: &ResolverInfo,
    ) -> bool {
        info.request.target.eq(alias_key)
    }

    pub(super) fn request_path_is_equal_alias_key_path(
        alias_path: &PathBuf,
        info: &ResolverInfo,
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
    fn apply(&self, resolver: &Resolver, info: ResolverInfo) -> ResolverStats {
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
                if should_deal_alias {
                    return match self
                        .deal_with_alias_fields_recursive(alias_target, &pkg_info.alias_fields)
                    {
                        AliasMap::Target(converted) => ResolverStats::Resolving(
                            info.with_path(pkg_info.abs_dir_path.to_path_buf())
                                .with_target(resolver, &converted),
                        ),
                        AliasMap::Ignored => ResolverStats::Success(ResolverResult::Ignored),
                    };
                }
            }
            ResolverStats::Resolving(info)
        } else {
            ResolverStats::Resolving(info)
        }
    }
}
