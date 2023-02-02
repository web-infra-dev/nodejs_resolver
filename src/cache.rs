use crate::fs::CachedFS;

#[derive(Debug, Default)]
pub struct Cache {
    pub fs: CachedFS,
}
