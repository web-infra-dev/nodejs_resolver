use crate::Resolver;

use super::{Plugin, ResolverInfo, ResolverStats};
#[derive(Default)]
pub struct PreferRelativePlugin;

impl Plugin for PreferRelativePlugin {
    fn apply(&self, resolver: &Resolver, info: ResolverInfo) -> ResolverStats {
        if info.request.target.starts_with("../") || info.request.target.starts_with("./") {
            return ResolverStats::Resolving(info);
        }

        if resolver.options.prefer_relative {
            let target = format!("./{}", info.request.target);
            let info = ResolverInfo::from(
                info.path.to_owned(),
                info.request.clone().with_target(&target),
            );
            let stats = resolver._resolve(info);
            if stats.is_success() {
                return stats;
            }
        }
        ResolverStats::Resolving(info)
    }
}
