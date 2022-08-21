use std::path::{Path, PathBuf};

use crate::Resolver;

impl Resolver {
    pub(super) fn find_up(now_dir: &Path, file_name: &str) -> Option<PathBuf> {
        let path = now_dir.join(file_name);
        if path.is_file() {
            Some(now_dir.to_path_buf())
        } else {
            now_dir
                .parent()
                .and_then(|parent| Self::find_up(parent, file_name))
        }
    }
}
