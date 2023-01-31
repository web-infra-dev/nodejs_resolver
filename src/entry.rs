use std::{
    io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
    time::SystemTime,
};

use crate::{
    description::{PkgInfo, PkgJSON},
    normalize::NormalizePath,
    RResult, Resolver,
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
        matches!(self, EntryKind::NonExist)
    }
}

#[derive(Debug, Clone)]
pub struct EntryStat {
    pub kind: EntryKind,
    pub mtime: Option<SystemTime>,
}

impl EntryStat {
    pub fn stat(path: &Path) -> io::Result<Self> {
        let stat = if let Ok(meta) = std::fs::metadata(path) {
            let kind = if meta.is_file() {
                EntryKind::File
            } else if meta.is_dir() {
                EntryKind::Dir
            } else {
                EntryKind::Unknown
            };
            let mtime = Some(meta.modified()?);
            EntryStat { kind, mtime }
        } else {
            EntryStat {
                kind: EntryKind::NonExist,
                mtime: None,
            }
        };
        Ok(stat)
    }
}

#[derive(Debug)]
pub struct Entry {
    pub parent: Option<Arc<Entry>>,
    pub path: PathBuf,
    pub pkg_info: Option<Arc<PkgInfo>>,
    pub stat: RwLock<Option<EntryStat>>,
    symlink: Mutex<CachedPath>,
}

#[derive(Debug, Clone, Default)]
pub enum CachedPath {
    #[default]
    Unresolved,
    NotSymlink,
    Resolved(Arc<Path>),
}

impl Entry {
    /// Returns the canonicalized path of `self.path` if it is a symlink.
    /// Returns None if `self.path` is not a symlink.
    pub fn symlink(&self) -> Option<Arc<Path>> {
        let mut cached_path = self.symlink.lock().unwrap().clone();

        if matches!(cached_path, CachedPath::Unresolved) {
            if self.path.read_link().is_err() {
                *self.symlink.lock().unwrap() = CachedPath::NotSymlink;
                return None;
            }
            cached_path = match dunce::canonicalize(&self.path) {
                Ok(symlink_path) => CachedPath::Resolved(Arc::from(symlink_path)),
                Err(_) => CachedPath::NotSymlink,
            };
            *self.symlink.lock().unwrap() = cached_path.clone();
        }

        match cached_path {
            CachedPath::Resolved(path) => Some(path),
            CachedPath::NotSymlink => None,
            CachedPath::Unresolved => unreachable!(),
        }
    }

    pub fn is_file(&self) -> bool {
        if let Some(stat) = self.stat.read().unwrap().as_ref() {
            return stat.kind.is_file();
        }
        if let Ok(stat) = EntryStat::stat(&self.path) {
            let is_file = stat.kind.is_file();
            let mut writer = self.stat.write().unwrap();
            *writer = Some(stat);
            is_file
        } else {
            false
        }
    }

    pub fn is_dir(&self) -> bool {
        if let Some(stat) = self.stat.read().unwrap().as_ref() {
            return stat.kind.is_dir();
        }
        if let Ok(stat) = EntryStat::stat(&self.path) {
            let is_dir = stat.kind.is_dir();
            let mut writer = self.stat.write().unwrap();
            *writer = Some(stat);
            is_dir
        } else {
            false
        }
    }

    pub fn is_exist(&self) -> bool {
        if let Some(stat) = self.stat.read().unwrap().as_ref() {
            return stat.kind.is_dir();
        }
        if let Ok(stat) = EntryStat::stat(&self.path) {
            let is_dir = stat.kind.exist();
            let mut writer = self.stat.write().unwrap();
            *writer = Some(stat);
            is_dir
        } else {
            false
        }
    }
}

impl Resolver {
    pub(super) fn load_entry(&self, path: &Path) -> RResult<Arc<Entry>> {
        let key = path.normalize();
        if let Some(cached) = self.entries.get(key.as_ref()) {
            Ok(cached.clone())
        } else {
            let entry = Arc::new(self.load_entry_uncached(&key)?);
            self.entries.entry(key.into()).or_insert(entry.clone());
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

        let pkg_name = &self.options.description_file;
        let is_pkg_name_suffix = path.ends_with(pkg_name);
        let maybe_pkg_path = if is_pkg_name_suffix {
            // path is xxxxx/xxxxx/package.json
            path.to_path_buf()
        } else {
            path.join(pkg_name)
        };

        let pkg_file_stat = EntryStat::stat(&maybe_pkg_path)?;
        let pkg_file_exist = pkg_file_stat.kind.is_file();

        let pkg_info = if pkg_file_exist {
            let content = self.cache.fs.read_file(&maybe_pkg_path, &pkg_file_stat)?;
            let pkg_json = if let Some(cached) = self.cache.pkg_json.get(&content) {
                cached.clone()
            } else {
                let result = Arc::new(PkgJSON::parse(&content, &maybe_pkg_path)?);
                self.cache.pkg_json.insert(content, result.clone());
                result
            };
            let dir_path = maybe_pkg_path.parent().unwrap().to_path_buf();
            let pkg_info = Arc::new(PkgInfo {
                json: pkg_json,
                dir_path,
            });
            Some(pkg_info)
        } else if let Some(parent) = &parent {
            parent.pkg_info.clone()
        } else {
            None
        };

        // Is path ended with `package.json`?
        // if `true`, then use above stats
        // else return `!true` means stat is None.
        let need_stat = !(pkg_info.is_some() && is_pkg_name_suffix);
        let stat = RwLock::new(if need_stat { None } else { Some(pkg_file_stat) });
        Ok(Entry {
            parent,
            path: path.to_path_buf(),
            pkg_info,
            stat,
            symlink: Mutex::new(CachedPath::default()),
        })
    }

    // TODO: should put entries as a parament.
    pub fn clear_entries(&self) {
        self.entries.clear();
    }

    #[must_use]
    pub fn get_dependency_from_entry(&self) -> (Vec<PathBuf>, Vec<PathBuf>) {
        let mut miss_dependency = vec![];
        let mut file_dependency = vec![];
        for entry in &self.entries {
            let reader = entry.as_ref().stat.read().unwrap();
            let kind = reader.as_ref().map(|reader| &reader.kind);
            if let Some(kind) = kind {
                if kind.is_file() || kind.is_dir() {
                    file_dependency.push(entry.path.clone());
                } else {
                    miss_dependency.push(entry.path.clone());
                }
            }
        }
        (file_dependency, miss_dependency)
    }
}

#[test]
fn dependency_test() {
    let case_path = super::test_helper::p(vec!["full", "a"]);
    let request = "package2";
    let resolver = Resolver::new(Default::default());
    resolver.resolve(&case_path, request).unwrap();
    let (file, missing) = resolver.get_dependency_from_entry();
    assert_eq!(file.len(), 3);
    assert_eq!(missing.len(), 1);
}
