use std::{
    fmt::Debug,
    fs, io,
    path::{Path, PathBuf},
};

#[async_trait::async_trait]
pub trait FileSystem: Sync + Send + Debug {
    async fn read_to_string(&self, path: &Path) -> io::Result<String>;
    async fn read_link(&self, path: &Path) -> io::Result<PathBuf>;
    async fn metadata(&self, path: &Path) -> io::Result<fs::Metadata>;
}
