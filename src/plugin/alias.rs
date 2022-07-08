use crate::{options::AliasMap, Resolver, MODULE};

use super::Plugin;
use crate::{ResolverInfo, ResolverResult, ResolverStats};

#[derive(Default)]
pub struct AliasPlugin;

impl Plugin for AliasPlugin {
    fn apply(&self, resolver: &Resolver, info: ResolverInfo) -> ResolverStats {
        let inner_target = &info.request.target;
        if info.path.display().to_string().contains(MODULE) {
            return ResolverStats::Resolving(info);
        }
        for (from, to) in &resolver.options.alias {
            if inner_target.starts_with(from) {
                match to {
                    AliasMap::Target(to) => {
                        if inner_target.eq(to) || inner_target.starts_with(to) {
                            // skip `target.eq(to)` to reduce useless function call.
                            // skip `target.starts_with(&format("{to}"))` to prevent infinite loop.
                            continue;
                        }
                        let normalized_target = inner_target.replacen(from, to, 1);
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
