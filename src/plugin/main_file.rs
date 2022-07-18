use std::sync::Arc;

use crate::{description::PkgFileInfo, Resolver};

use super::{
    AliasFieldPlugin, ExportsFieldPlugin, ImportsFieldPlugin, Plugin, ResolveInfo, ResolverStats,
};

pub struct MainFilePlugin<'a> {
    pkg_info: &'a Option<Arc<PkgFileInfo>>,
}

impl<'a> MainFilePlugin<'a> {
    pub fn new(pkg_info: &'a Option<Arc<PkgFileInfo>>) -> Self {
        Self { pkg_info }
    }
}

impl<'a> Plugin for MainFilePlugin<'a> {
    fn apply(&self, resolver: &Resolver, info: ResolveInfo) -> ResolverStats {
        let mut main_file_info = ResolveInfo::from(info.path.to_owned(), info.request.clone());
        for main_file in &resolver.options.main_files {
            main_file_info = main_file_info.with_target(&format!("./{main_file}"));
            let stats = ExportsFieldPlugin::new(self.pkg_info)
                .apply(resolver, main_file_info)
                .and_then(|info| ImportsFieldPlugin::new(self.pkg_info).apply(resolver, info))
                .and_then(|info| AliasFieldPlugin::new(self.pkg_info).apply(resolver, info))
                .and_then(|info| resolver.resolve_as_file(info));
            if stats.is_success() {
                return stats;
            } else {
                main_file_info = stats.extract_info();
            }
        }
        ResolverStats::Resolving(info)
    }
}
