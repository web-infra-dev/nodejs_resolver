use crate::description::PkgInfoInner;
use crate::fs::CachedFS;
use dashmap::DashMap;
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct ResolverCache {
    pub fs: CachedFS,
    pub pkg_info: CachedPkgInfo,
}

type Content = String;
type CachedPkgInfo = DashMap<Content, Arc<PkgInfoInner>>;
