use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::SystemTime,
};

use crate::{
    description::{PkgInfo, PkgJSON},
    Error, RResult, Resolver,
};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

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
    pub fn stat(path: &Path) -> std::io::Result<Self> {
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
    pub symlink: RwLock<Option<PathBuf>>,
}

impl Entry {
    pub fn symlink(&self) -> std::io::Result<PathBuf> {
        if let Some(symlink) = self.symlink.read().unwrap().as_ref() {
            return Ok(symlink.to_path_buf());
        }
        let real_path = std::fs::canonicalize(&self.path)?;
        let mut writer = self.symlink.write().unwrap();
        *writer = Some(real_path.clone());
        Ok(real_path)
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

    #[cfg(windows)]
    fn has_trailing_slash(p: &Path) -> bool {
        let last = p.as_os_str().encode_wide().last();
        last == Some(b'\\' as u16) || last == Some(b'/' as u16)
    }
    #[cfg(unix)]
    fn has_trailing_slash(p: &Path) -> bool {
        p.as_os_str().as_bytes().last() == Some(&b'/')
    }

    pub fn path_to_key(path: &Path) -> (PathBuf, bool) {
        (path.to_path_buf(), Self::has_trailing_slash(path))
    }
}

impl Resolver {
    pub(super) fn load_entry(&self, path: &Path) -> RResult<Arc<Entry>> {
        let key = Entry::path_to_key(path);
        if let Some(cached) = self.entries.get(&key) {
            // tracing::debug!(
            //     "Load entry '{}' from cache",
            //     log::color::blue(&cached.path.display())
            // );
            Ok(cached.clone())
        } else {
            // TODO: how to mutex that?
            let entry = Arc::new(self.load_entry_uncached(path)?);
            // tracing::debug!(
            //     "Load entry '{}' missing cache",
            //     log::color::red(&entry.path.display())
            // );
            self.entries.entry(key).or_insert(entry.clone());
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

        let pkg_file_stat = EntryStat::stat(&maybe_pkg_path).map_err(Error::Io)?;
        let pkg_file_exist = pkg_file_stat.kind.is_file();
        let pkg_info = if pkg_file_exist {
            let content = self
                .cache
                .fs
                .read_file(&maybe_pkg_path, &pkg_file_stat)
                .map_err(Error::Io)?;
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
        let symlink = RwLock::new(None);
        Ok(Entry {
            parent,
            path: path.to_path_buf(),
            pkg_info,
            stat,
            symlink,
        })
    }

    // TODO: should put entries as a parament.
    pub fn clear_entries(&self) {
        self.entries.clear();
    }

    pub fn get_dependency_from_entry(&self) -> (Vec<PathBuf>, Vec<PathBuf>) {
        let mut miss_dependency = vec![];
        let mut file_dependency = vec![];
        for entry in &self.entries {
            let reader = entry.as_ref().stat.read().unwrap();
            let kind = reader.as_ref().map(|reader| &reader.kind);
            if let Some(kind) = kind {
                if kind.is_file() || kind.is_dir() {
                    file_dependency.push(entry.path.to_path_buf())
                } else {
                    miss_dependency.push(entry.path.to_path_buf())
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
