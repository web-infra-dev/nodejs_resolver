use std::path::Path;

use crate::{RResult, Resolver};

impl Resolver {
    pub(super) fn raise<T>(base_dir: &Path, target: &str) -> RResult<T> {
        if target.is_empty() {
            Err(format!("Resolve '' failed in {}", base_dir.display()))
        } else {
            Err(format!("Resolve {target} failed in {}", base_dir.display()))
        }
    }
}
