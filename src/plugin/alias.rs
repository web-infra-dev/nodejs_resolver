use crate::{options::AliasMap, Resolver};

use super::Plugin;
use crate::{ResolverInfo, ResolverResult, ResolverStats};

#[derive(Default)]
pub struct AliasPlugin;

impl Plugin for AliasPlugin {
    fn apply(&self, resolver: &Resolver, info: ResolverInfo) -> ResolverStats {
        let target = &info.request.target;
        for (from, to) in &resolver.options.alias {
            if target.starts_with(from) {
                match to {
                    AliasMap::Target(to) => {
                        let normalized_target = target.replacen(from, to, 1);
                        let alias_info = ResolverInfo::from(
                            info.path.to_path_buf(),
                            info.request
                                .clone()
                                .with_target(resolver, &normalized_target),
                        );
                        let stats = resolver._resolve(alias_info);
                        if stats.is_success() {
                            return stats;
                        }
                    }
                    AliasMap::Ignored => return ResolverStats::Success(ResolverResult::Ignored),
                }
            }
        }

        ResolverStats::Resolving(info)
    }
}
