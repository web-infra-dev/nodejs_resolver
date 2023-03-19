use crate::{
    description::{DescriptionData, PkgJSON},
    entry::EntryStat,
    tsconfig::TsConfig,
    RResult,
};
use rustc_hash::FxHasher;
use std::{
    fmt::Debug,
    fs,
    hash::BuildHasherDefault,
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

use dashmap::DashMap;

use std::time::Duration;

#[derive(Debug, Default)]
pub struct CachedFS {
    /// Caches raw files
    entries: CachedMap<String>,

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
        Self {
            content: content.into(),
            stat,
        }
    }

    fn content(&self) -> Arc<T> {
        self.content.clone()
    }
}

const DEBOUNCE_INTERVAL: Duration = Duration::from_millis(300);

impl CachedFS {
    pub fn read_file(&self, path: &Path, file_stat: EntryStat) -> RResult<Arc<String>> {
        if let Some(cached) = self.entries.get(path) {
            if self.is_modified(file_stat.modified(), cached.stat.modified()) {
                return Ok(cached.value().content());
            }
        }
        let string = fs::read_to_string(path)?;
        let entry = CachedEntry::new(string, file_stat);
        self.entries.insert(path.to_path_buf(), entry.clone());
        Ok(entry.content())
    }

    pub fn read_description_file(
        &self,
        path: &Path,
        file_stat: EntryStat,
    ) -> RResult<Arc<DescriptionData>> {
        if let Some(cached) = self.descriptions.get(path) {
            if self.is_modified(file_stat.modified(), cached.stat.modified()) {
                return Ok(cached.value().content());
            }
        }
        let string = fs::read_to_string(path)?;
        let json = PkgJSON::parse(&string, path)?;
        let dir = path.parent().unwrap().to_path_buf();
        let info = DescriptionData::new(json, dir);
        let entry = CachedEntry::new(info, file_stat);
        self.descriptions.insert(path.to_path_buf(), entry.clone());
        Ok(entry.content())
    }

    pub fn read_tsconfig(
        &self,
        path: &Path,
        file_stat: EntryStat,
    ) -> RResult<Arc<serde_json::Value>> {
        if let Some(cached) = self.tsconfigs.get(path) {
            if self.is_modified(file_stat.modified(), cached.stat.modified()) {
                return Ok(cached.value().content());
            }
        }
        let string = fs::read_to_string(path)?;
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
