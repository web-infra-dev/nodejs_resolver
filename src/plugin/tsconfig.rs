use super::Plugin;
use crate::{Resolver, ResolverInfo, ResolverStats, TsConfigInfo};
use std::sync::Arc;

pub struct TsConfigPathPlugin;

impl TsConfigPathPlugin {
    fn load_config(&self, resolver: &Resolver) -> Arc<TsConfigInfo> {
        Arc::new(TsConfigInfo {})
    }
}

impl Plugin for TsConfigPathPlugin {
    fn apply(&self, resolver: &Resolver, info: ResolverInfo) -> ResolverStats {
        ResolverStats::Resolving(info)
    }
}
