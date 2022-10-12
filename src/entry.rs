use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::SystemTime,
};

use crate::{
    description::{PkgInfo, PkgJSON},
    fs::CachedFS,
    RResult, Resolver, ResolverError,
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
}

#[derive(Debug, Clone)]
pub struct EntryStat {
    pub kind: EntryKind,
    pub mtime: Option<SystemTime>,
}

#[derive(Debug)]
pub struct Entry {
    pub parent: Option<Arc<Entry>>,
    path: PathBuf,
    pub pkg_info: Option<Arc<PkgInfo>>,
    stat: Mutex<Option<EntryStat>>,
    symlink: Mutex<Option<PathBuf>>,
}

impl Entry {
    pub fn symlink(&self) -> std::io::Result<PathBuf> {
        let mut value = self.symlink.lock().unwrap();
        match value.as_ref() {
            Some(symlink) => Ok(symlink.to_path_buf()),
            None => {
                let real_path = std::fs::canonicalize(&self.path)?;
                *value = Some(real_path.clone());
                Ok(real_path)
            }
        }
    }

    pub fn is_file(&self) -> bool {
        let mut value = self.stat.lock().unwrap();
        match value.as_ref() {
            Some(stat) => stat.kind.is_file(),
            None => {
                if let Ok(stat) = CachedFS::stat(&self.path) {
                    let is_file = stat.kind.is_file();
                    *value = Some(stat);
                    is_file
                } else {
                    false
                }
            }
        }
    }

    pub fn is_dir(&self) -> bool {
        let mut value = self.stat.lock().unwrap();
        match value.as_ref() {
            Some(stat) => stat.kind.is_dir(),
            None => {
                if let Ok(stat) = CachedFS::stat(&self.path) {
                    let is_dir = stat.kind.is_dir();
                    *value = Some(stat);
                    is_dir
                } else {
                    false
                }
            }
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
            Ok(cached.clone())
        } else {
            let entry = self.load_entry_uncached(path)?;
            let entry = Arc::new(entry);
            self.entries.insert(key, entry.clone());
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
        let pkg_file_stat = CachedFS::stat(&maybe_pkg_path).map_err(ResolverError::Io)?;
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
            Some(pkg_info)
        } else if let Some(parent) = &parent {
            parent.pkg_info.clone()
        } else {
            None
        };

        let need_stat = if let Some(info) = &pkg_info {
            // Is path pointed xxx.package.json ?
            // if `true`, then use above stats
            // else return `!true` means stat is None.
            let is_pkg_file = info.dir_path.join(&pkg_file_name).eq(&path);
            !is_pkg_file
        } else {
            true
        };

        let stat = Mutex::new(if need_stat { None } else { Some(pkg_file_stat) });
        let symlink = Mutex::new(None);
        Ok(Entry {
            parent,
            path,
            pkg_info,
            stat,
            symlink,
        })
    }

    // TODO: should put entries as a parament.
    pub fn clear_entries(&self) {
        self.entries.clear();
    }
}
