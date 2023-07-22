use std::{hash::BuildHasherDefault, path::Path, sync::Arc};

use rustc_hash::FxHasher;

use crate::entry::Entry;
use crate::fs::CachedFS;

#[derive(Debug, Default)]
pub struct Cache {
    pub fs: CachedFS,
    /// File entries keyed by normalized paths
    pub entries: dashmap::DashMap<Box<Path>, Arc<Entry>, BuildHasherDefault<FxHasher>>,
}
