use crate::description::PkgJSON;
use crate::fs::CachedFS;
use dashmap::DashMap;
use rustc_hash::FxHasher;
use std::{hash::BuildHasherDefault, sync::Arc};

#[derive(Debug, Default)]
pub struct Cache {
    pub fs: CachedFS,
    pub pkg_json: CachedPkgJSON,
}

type Content = String;
type CachedPkgJSON = DashMap<Content, Arc<PkgJSON>, BuildHasherDefault<FxHasher>>;
