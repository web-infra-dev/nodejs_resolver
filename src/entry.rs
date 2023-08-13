use std::path::{Path, PathBuf};
use std::{borrow::Cow, fs::FileType, sync::Arc, time::SystemTime};

use async_once_cell::OnceCell;

use crate::{description::DescriptionData, Error, RResult, Resolver};

#[derive(Debug, Default, Clone, Copy)]
pub struct EntryStat {
    /// `None` for non-existing file
    file_type: Option<FileType>,

    /// `None` for existing file but without system time.
    modified: Option<SystemTime>,
}

impl EntryStat {
    fn new(file_type: Option<FileType>, modified: Option<SystemTime>) -> Self {
        Self { file_type, modified }
    }

    /// Returns `None` for non-existing file
    pub fn file_type(&self) -> Option<FileType> {
        self.file_type
    }

    /// Returns `None` for existing file but without system time.
    pub fn modified(&self) -> Option<SystemTime> {
        self.modified
    }
}

#[derive(Debug)]
pub struct Entry {
    parent: Option<Arc<Entry>>,
    path: Box<Path>,
    // None: package.json does not exist
    pkg_info: OnceCell<Option<Arc<DescriptionData>>>,
    stat: OnceCell<EntryStat>,
    /// None represent the `self.path` is not a symlink
    symlink: OnceCell<Option<Box<Path>>>,
    /// If `self.path` is a symlink, then return canonicalized path,
    /// else return `self.path`
    real: std::sync::OnceLock<Box<Path>>,
}

impl Entry {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn parent(&self) -> Option<&Arc<Entry>> {
        self.parent.as_ref()
    }

    pub fn real(&self) -> Option<&Path> {
        self.real.get().map(|p| &**p)
    }

    pub fn init_real(&self, path: Box<Path>) {
        self.real.get_or_init(|| path);
    }
}

impl Resolver {
    pub(super) fn load_entry(&self, path: &Path) -> Arc<Entry> {
        if let Some(cached) = self.cache.entries.get(path) {
            cached.clone()
        } else {
            let entry = Arc::new(self.load_entry_uncached(path));
            self.cache.entries.entry(path.into()).or_insert(entry.clone());
            entry
        }
    }

    fn load_entry_uncached(&self, path: &Path) -> Entry {
        let parent = if let Some(parent) = path.parent() {
            let entry = self.load_entry(parent);
            Some(entry)
        } else {
            None
        };
        Entry {
            parent,
            path: path.into(),
            pkg_info: OnceCell::default(),
            stat: OnceCell::default(),
            symlink: OnceCell::default(),
            real: Default::default(),
        }
    }

    async fn stat(&self, path: &Path) -> EntryStat {
        if let Ok(meta) = self.fs.metadata(path).await {
            // This field might not be available on all platforms,
            // and will return an Err on platforms where it is not available.
            let modified = meta.modified().ok();
            EntryStat::new(Some(meta.file_type()), modified)
        } else {
            EntryStat::new(None, None)
        }
    }

    // TODO: should put entries as a parament.
    pub fn clear_entries(&self) {
        self.cache.entries.clear();
    }

    #[must_use]
    pub fn get_dependency_from_entry(&self) -> (Vec<PathBuf>, Vec<PathBuf>) {
        todo!("get_dependency_from_entry")
    }

    #[async_recursion::async_recursion]
    pub async fn pkg_info<'a>(
        &'a self,
        entry: &'a Entry,
    ) -> RResult<&'a Option<Arc<DescriptionData>>> {
        let future = entry.pkg_info.get_or_try_init(async {
            let pkg_name = &self.options.description_file;
            let path = entry.path();
            let is_pkg_suffix = path.ends_with(pkg_name);
            if self.is_dir(entry).await || is_pkg_suffix {
                let pkg_path = if is_pkg_suffix {
                    Cow::Borrowed(path)
                } else {
                    Cow::Owned(path.join(pkg_name))
                };
                match self
                    .cache
                    .fs
                    .read_description_file(self, &pkg_path, EntryStat::default())
                    .await
                {
                    Ok(info) => {
                        return Ok(Some(info));
                    }
                    Err(error @ (Error::UnexpectedJson(_) | Error::UnexpectedValue(_))) => {
                        // Return bad json
                        return Err(error);
                    }
                    Err(Error::Io(_)) => {
                        // package.json not found
                    }
                    _ => unreachable!(),
                };
            }
            if let Some(parent) = &entry.parent() {
                return self.pkg_info(parent).await.cloned();
            }
            Ok(None)
        });
        future.await
    }

    pub async fn is_file(&self, entry: &Entry) -> bool {
        self.cached_stat(entry).await.file_type().map_or(false, |ft| ft.is_file())
    }

    pub async fn is_dir(&self, entry: &Entry) -> bool {
        self.cached_stat(entry).await.file_type().map_or(false, |ft| ft.is_dir())
    }

    pub async fn exists(&self, entry: &Entry) -> bool {
        self.cached_stat(entry).await.file_type().is_some()
    }

    pub async fn cached_stat(&self, entry: &Entry) -> EntryStat {
        *entry.stat.get_or_init(async { self.stat(&entry.path).await }).await
    }

    /// Returns the canonicalized path of `self.path` if it is a symlink.
    /// Returns None if `self.path` is not a symlink.
    pub async fn symlink<'a>(&'a self, entry: &'a Entry) -> &'a Option<Box<Path>> {
        entry
            .symlink
            .get_or_init(async {
                assert!(entry.path.is_absolute());
                if self.fs.read_link(&entry.path).await.is_err() {
                    return None;
                }
                match dunce::canonicalize(&entry.path) {
                    Ok(symlink_path) => Some(Box::from(symlink_path)),
                    Err(_) => None,
                }
            })
            .await
    }
}

// #[test]
// #[ignore]
// fn dependency_test() {
//     let case_path = super::test_helper::p(vec!["full", "a"]);
//     let request = "package2";
//     let resolver = Resolver::new(Default::default());
//     resolver.resolve(&case_path, request).unwrap();
//     let (file, missing) = resolver.get_dependency_from_entry();
//     assert_eq!(file.len(), 3);
//     assert_eq!(missing.len(), 1);
// }
