use crate::description::PkgJSON;
use crate::fs::CachedFS;
use dashmap::DashMap;
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct ResolverCache {
    pub fs: CachedFS,
    pub pkg_json: CachedPkgJSON,
}

type Content = String;
type CachedPkgJSON = DashMap<Content, Arc<PkgJSON>>;
