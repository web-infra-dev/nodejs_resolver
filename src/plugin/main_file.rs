use std::sync::Arc;

use crate::{description::PkgFileInfo, Resolver};

use super::{Plugin, ResolverInfo, ResolverStats};

pub struct MainFilePlugin<'a> {
    pkg_info: &'a Option<Arc<PkgFileInfo>>,
}

impl<'a> MainFilePlugin<'a> {
    pub fn new(pkg_info: &'a Option<Arc<PkgFileInfo>>) -> Self {
        Self { pkg_info }
    }
}

impl<'a> Plugin for MainFilePlugin<'a> {
    fn apply(&self, resolver: &Resolver, info: ResolverInfo) -> ResolverStats {
        // TODO: should optimized.
        let mut main_file_info = ResolverInfo::from(info.path.to_owned(), info.request.clone());
        for main_file in &resolver.options.main_files {
            main_file_info = main_file_info.with_target(resolver, &format!("./{main_file}"));
            let stats = resolver
                .get_real_target(main_file_info, self.pkg_info)
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
