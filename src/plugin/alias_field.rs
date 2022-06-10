use crate::{description::PkgFileInfo, AliasMap, Resolver};
use std::collections::HashMap;
use std::sync::Arc;

use super::Plugin;
use crate::{PathKind, ResolverInfo, ResolverResult, ResolverStats};

pub struct AliasFieldPlugin<'a> {
    pkg_info: &'a Arc<PkgFileInfo>,
}

impl<'a> AliasFieldPlugin<'a> {
    pub fn new(pkg_info: &'a Arc<PkgFileInfo>) -> Self {
        Self { pkg_info }
    }

    pub(crate) fn deal_with_alias_fields_recursive(
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
}

impl<'a> Plugin for AliasFieldPlugin<'a> {
    fn apply(&self, resolver: &Resolver, info: ResolverInfo) -> ResolverStats {
        let description_file_dir = &self.pkg_info.abs_dir_path;
        let path = info.get_path();

        for (relative_path, converted_target) in &self.pkg_info.alias_fields {
            // TODO: should optimized.
            if matches!(info.request.kind, PathKind::Normal | PathKind::Internal)
                && info.request.target.eq(relative_path)
            {
                return match self
                    .deal_with_alias_fields_recursive(converted_target, &self.pkg_info.alias_fields)
                {
                    AliasMap::Target(converted) => ResolverStats::Resolving(
                        info.with_path(description_file_dir.to_path_buf())
                            .with_target(resolver, &converted),
                    ),
                    AliasMap::Ignored => ResolverStats::Success(ResolverResult::Ignored),
                };
            }

            let should_converted_path = description_file_dir.join(relative_path);
            // TODO: should optimized.
            if should_converted_path.eq(&path)
                || resolver.options.extensions.iter().any(|ext| {
                    let path_with_extension = Resolver::append_ext_for_path(&path, ext);
                    should_converted_path.eq(&path_with_extension)
                })
            {
                return match self
                    .deal_with_alias_fields_recursive(converted_target, &self.pkg_info.alias_fields)
                {
                    AliasMap::Target(converted) => ResolverStats::Resolving(
                        info.with_path(description_file_dir.to_path_buf())
                            .with_target(resolver, &converted),
                    ),
                    AliasMap::Ignored => ResolverStats::Success(ResolverResult::Ignored),
                };
            }
        }
        ResolverStats::Resolving(info)
    }
}
