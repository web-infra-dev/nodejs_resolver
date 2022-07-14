use crate::description::PkgFileInfo;
use crate::map::PathTreeNode;
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Default, Debug, Clone)]
pub struct ResolverCache {
    /// file_directory -> the closet package.json info
    pub file_dir_to_pkg_info: DashMap<PathBuf, Option<Arc<PkgFileInfo>>>,
    pub exports_content_to_tree: DashMap<String, Arc<PathTreeNode>>,
    pub imports_content_to_tree: DashMap<String, Arc<PathTreeNode>>,
}

#[derive(Default, Debug, Clone)]
pub(crate) struct DebugReadMap(DashMap<PathBuf, bool>);

impl DebugReadMap {
    pub(crate) fn remove(&self, path: &Path) {
        self.0.remove(path);
    }

    pub(crate) fn contains_key(&self, path: &Path) -> bool {
        self.0.contains_key(path)
    }

    pub(crate) fn insert(&self, path: &Path, value: bool) {
        self.0.insert(path.to_path_buf(), value);
    }
}
