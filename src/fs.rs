use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::{hash::BuildHasherDefault, sync::Arc, time::SystemTime};

use dashmap::DashMap;
use rustc_hash::FxHasher;

use crate::description::{DescriptionData, PkgJSON};
use crate::Resolver;
use crate::{entry::EntryStat, tsconfig::TsConfig, RResult};

#[derive(Debug, Default)]
pub struct CachedFS {
    /// Caches parsed package.json
    descriptions: CachedMap<DescriptionData>,

    /// Caches tsconfig.json
    tsconfigs: CachedMap<serde_json::Value>,
}

pub type CachedMap<T> = DashMap<PathBuf, CachedEntry<T>, BuildHasherDefault<FxHasher>>;

#[derive(Debug, Clone)]
pub struct CachedEntry<T: Sized> {
    content: Arc<T>,
    stat: EntryStat,
}

impl<T: Sized> CachedEntry<T> {
    fn new(content: T, stat: EntryStat) -> Self {
        Self { content: content.into(), stat }
    }

    fn content(&self) -> Arc<T> {
        self.content.clone()
    }
}

const DEBOUNCE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(300);

impl CachedFS {
    pub async fn read_description_file(
        &self,
        resolver: &Resolver,
        path: &Path,
        file_stat: EntryStat,
    ) -> RResult<Arc<DescriptionData>> {
        if let Some(cached) = self.descriptions.get(path) {
            if self.is_modified(file_stat.modified(), cached.stat.modified()) {
                return Ok(cached.value().content());
            }
        }
        let string = resolver.fs.read_to_string(path).await?;
        let json = PkgJSON::parse(&string, path)?;
        let dir = path.parent().unwrap().to_path_buf();
        let info = DescriptionData::new(json, dir);
        let entry = CachedEntry::new(info, file_stat);
        self.descriptions.insert(path.to_path_buf(), entry.clone());
        Ok(entry.content())
    }

    pub async fn read_tsconfig(
        &self,
        resolver: &Resolver,
        path: &Path,
        file_stat: EntryStat,
    ) -> RResult<Arc<serde_json::Value>> {
        if let Some(cached) = self.tsconfigs.get(path) {
            if self.is_modified(file_stat.modified(), cached.stat.modified()) {
                return Ok(cached.value().content());
            }
        }
        let string = resolver.fs.read_to_string(path).await?;
        let serde_json = TsConfig::parse(&string, path)?;
        let entry = CachedEntry::new(serde_json, file_stat);
        self.tsconfigs.insert(path.to_path_buf(), entry.clone());
        Ok(entry.content())
    }

    fn is_modified(&self, before: Option<SystemTime>, after: Option<SystemTime>) -> bool {
        if let (Some(before), Some(after)) = (before, after) {
            if before.duration_since(after).expect("after > before") < DEBOUNCE_INTERVAL {
                return true;
            }
        }
        false
    }
}
