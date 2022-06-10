use std::sync::Arc;

use crate::{description::PkgFileInfo, Resolver};

use super::{AliasFieldPlugin, Plugin, ResolverInfo, ResolverStats};

pub struct MainFieldPlugin<'a> {
    pkg_info: &'a Option<Arc<PkgFileInfo>>,
}

impl<'a> MainFieldPlugin<'a> {
    pub fn new(pkg_info: &'a Option<Arc<PkgFileInfo>>) -> Self {
        Self { pkg_info }
    }
}

impl<'a> Plugin for MainFieldPlugin<'a> {
    fn apply(&self, resolver: &Resolver, info: ResolverInfo) -> ResolverStats {
        if let Some(pkg_info) = self.pkg_info {
            if !info.path.eq(&pkg_info.abs_dir_path) {
                return ResolverStats::Resolving(info);
            }
            let mut main_field_info =
                ResolverInfo::from(info.path.to_owned(), info.request.clone());
            for main_field in &pkg_info.main_fields {
                if main_field == "." || main_field == "./" {
                    // if it pointed to itself.
                    break;
                }
                main_field_info = main_field_info.with_target(resolver, main_field);
                let stats = resolver
                    .deal_imports_exports_field_plugin(main_field_info, pkg_info)
                    .and_then(|info| AliasFieldPlugin::new(pkg_info).apply(resolver, info))
                    .and_then(|info| resolver.resolve_as_file(info))
                    .and_then(|info| resolver.resolve_as_dir(info));

                if stats.is_success() {
                    return stats;
                } else {
                    main_field_info = stats.extract_info();
                }
            }
        }
        ResolverStats::Resolving(info)
    }
}
