use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::SystemTime,
};

use crate::{
    description::{PkgInfo, PkgJSON},
    RResult, Resolver, ResolverError,
};

#[derive(Debug, Clone)]
pub enum EntryKind {
    File,
    Dir,
    NonExist,
    Unknown,
}

impl EntryKind {
    pub fn is_file(&self) -> bool {
        matches!(self, EntryKind::File)
    }

    pub fn is_dir(&self) -> bool {
        matches!(self, EntryKind::Dir)
    }

    pub fn exist(&self) -> bool {
        !matches!(self, EntryKind::NonExist)
    }
}

#[derive(Debug, Clone)]
pub struct EntryStat {
    pub kind: EntryKind,
    pub mtime: Option<SystemTime>,
}

#[derive(Debug)]
pub struct Entry {
    parent: Option<Arc<Entry>>,
    path: PathBuf,
    pub pkg_info: Option<Arc<PkgInfo>>,
    need_stat: bool,
    mutex: Mutex<()>,
    stat: Option<EntryStat>,
    symlink: Option<PathBuf>,
}

impl Entry {
    pub fn exist(&self) -> bool {
        false
    }

    pub fn is_file(&self) -> bool {
        false
    }

    pub fn is_dir(&self) -> bool {
        false
    }
}

impl Resolver {
    pub(super) fn load_entry(&self, path: &Path) -> RResult<Arc<Entry>> {
        if let Some(cached) = self.entries.get(path) {
            Ok(cached.clone())
        } else {
            let entry = self.load_entry_uncached(path)?;
            let entry = Arc::new(entry);
            self.entries.insert(path.to_path_buf(), entry.clone());
            Ok(entry)
        }
    }

    fn load_entry_uncached(&self, path: &Path) -> RResult<Entry> {
        let parent = if let Some(parent) = path.parent() {
            let entry = self.load_entry(parent)?;
            Some(entry)
        } else {
            None
        };
        let path = path.to_path_buf();
        let pkg_file_name = &self.options.description_file;
        let maybe_pkg_path = path.join(pkg_file_name);
        let pkg_file_stat = self
            .cache
            .fs
            .stat(&maybe_pkg_path)
            .map_err(ResolverError::Io)?;
        let pkg_info = if pkg_file_stat.kind.is_file() {
            let content = self
                .cache
                .fs
                .read_file(&maybe_pkg_path, &pkg_file_stat)
                .map_err(ResolverError::Io)?;
            let pkg_json = if let Some(cached) = self.cache.pkg_json.get(&content) {
                cached.clone()
            } else {
                Arc::new(PkgJSON::parse(&content, &maybe_pkg_path)?)
            };
            let dir_path = path.clone();
            self.cache.pkg_json.insert(content, pkg_json.clone());
            let pkg_info = Arc::new(PkgInfo {
                json: pkg_json,
                dir_path,
            });
            Some(pkg_info.clone())
        } else if let Some(parent) = &parent {
            parent.pkg_info.clone()
        } else {
            None
        };

        let need_stat = if let Some(info) = &pkg_info {
            info.dir_path.join(&pkg_file_name).eq(&path)
        } else {
            false
        };

        let stat = if need_stat { None } else { Some(pkg_file_stat) };
        let mutex = Default::default();
        Ok(Entry {
            mutex,
            parent,
            path,
            pkg_info,
            need_stat,
            stat,
            symlink: None,
        })
    }

    // TODO: should put entries as a parament.
    pub fn clear_entries(&self) {
        self.entries.clear();
    }
}
