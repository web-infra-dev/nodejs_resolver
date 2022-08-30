use crate::{ResolveInfo, Resolver, ResolverStats};

use super::Plugin;

pub struct MainFilePlugin;

impl Plugin for MainFilePlugin {
    fn apply(&self, resolver: &Resolver, info: ResolveInfo) -> ResolverStats {
        let mut main_file_info = ResolveInfo::from(info.path.to_owned(), info.request.clone());
        for main_file in &resolver.options.main_files {
            main_file_info = main_file_info.with_target(&format!("./{main_file}"));
            let stats = resolver._resolve(main_file_info);
            if stats.is_success() {
                return stats;
            } else {
                main_file_info = stats.extract_info();
            }
        }
        ResolverStats::Resolving(info)
    }
}
