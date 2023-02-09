use std::{
    fs::FileType,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
    time::SystemTime,
};

use crate::{
    description::{PkgInfo, PkgJSON},
    normalize::NormalizePath,
    RResult, Resolver,
};

#[derive(Debug, Clone, Copy)]
pub struct EntryStat {
    /// `None` for non-existing file
    file_type: Option<FileType>,

    /// `None` for existing file but without system time.
    modified: Option<SystemTime>,
}

impl EntryStat {
    fn new(file_type: Option<FileType>, modified: Option<SystemTime>) -> Self {
        Self {
            file_type,
            modified,
        }
    }

    /// Returns `None` for non-existing file
    pub fn file_type(&self) -> Option<FileType> {
        self.file_type
    }

    /// Returns `None` for existing file but without system time.
    pub fn modified(&self) -> Option<SystemTime> {
        self.modified
    }

    fn stat(path: &Path) -> Self {
        if !path.is_absolute() {
            Self::new(None, None)
        } else if let Ok(meta) = path.metadata() {
            // This field might not be available on all platforms,
            // and will return an Err on platforms where it is not available.
            let modified = meta.modified().ok();
            Self::new(Some(meta.file_type()), modified)
        } else {
            Self::new(None, None)
        }
    }
}

#[derive(Debug)]
pub struct Entry {
    parent: Option<Arc<Entry>>,
    path: Box<Path>,
    pkg_info: Option<Arc<PkgInfo>>,
    stat: RwLock<Option<EntryStat>>,
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
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn parent(&self) -> Option<&Arc<Entry>> {
        self.parent.as_ref()
    }

    pub fn pkg_info(&self) -> Option<&Arc<PkgInfo>> {
        self.pkg_info.as_ref()
    }

    pub fn is_file(&self) -> bool {
        self.cached_stat()
            .file_type()
            .map_or(false, |ft| ft.is_file())
    }

    pub fn is_dir(&self) -> bool {
        self.cached_stat()
            .file_type()
            .map_or(false, |ft| ft.is_dir())
    }

    pub fn exists(&self) -> bool {
        self.cached_stat().file_type().is_some()
    }

    pub fn cached_stat(&self) -> EntryStat {
        let stat = *self.stat.read().unwrap();
        if let Some(stat) = stat {
            return stat;
        }
        let stat = EntryStat::stat(&self.path);
        *self.stat.write().unwrap() = Some(stat);
        stat
    }

    /// Returns the canonicalized path of `self.path` if it is a symlink.
    /// Returns None if `self.path` is not a symlink.
    pub fn symlink(&self) -> Option<Arc<Path>> {
        let mut cached_path = self.symlink.lock().unwrap().clone();
        if matches!(cached_path, CachedPath::Unresolved) {
            if !self.exists() || self.path.read_link().is_err() {
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

        let pkg_file_stat = EntryStat::stat(&maybe_pkg_path);
        let pkg_file_exist = pkg_file_stat.file_type().map_or(false, |ft| ft.is_file());

        let pkg_info = if pkg_file_exist {
            let content = self.cache.fs.read_file(&maybe_pkg_path, &pkg_file_stat)?;
            let pkg_json = if let Some(cached) = self.cache.pkg_json.get(&content) {
                cached.clone()
            } else {
                let result = Arc::new(PkgJSON::parse(&content, &maybe_pkg_path)?);
                self.cache.pkg_json.insert(content, result.clone());
                result
            };
            let dir_path = maybe_pkg_path.parent().unwrap().into();
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
        Ok(Entry {
            parent,
            path: path.into(),
            pkg_info,
            stat: RwLock::new(if need_stat { None } else { Some(pkg_file_stat) }),
            symlink: Mutex::new(CachedPath::default()),
        })
    }

    // TODO: should put entries as a parament.
    pub fn clear_entries(&self) {
        self.entries.clear();
    }

    #[must_use]
    pub fn get_dependency_from_entry(&self) -> (Vec<PathBuf>, Vec<PathBuf>) {
        todo!("get_dependency_from_entry")
    }
}

#[test]
#[ignore]
fn dependency_test() {
    let case_path = super::test_helper::p(vec!["full", "a"]);
    let request = "package2";
    let resolver = Resolver::new(Default::default());
    resolver.resolve(&case_path, request).unwrap();
    let (file, missing) = resolver.get_dependency_from_entry();
    assert_eq!(file.len(), 3);
    assert_eq!(missing.len(), 1);
}
