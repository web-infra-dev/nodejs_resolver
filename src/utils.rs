use std::path::{Path, PathBuf};

use crate::{Resolver, ResolverInfo};

pub static RAISE_RESOLVE_ERROR_TAG: &str = "T0";

impl Resolver {
    pub(super) fn raise_tag() -> String {
        RAISE_RESOLVE_ERROR_TAG.to_string()
    }

    pub(super) fn raise_resolve_failed_message(info: &ResolverInfo) -> String {
        format!(
            "Resolve '{}' failed in '{}'",
            info.request,
            info.path.display()
        )
    }

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
