mod kind;
mod normalize;
mod options;
mod parse;
mod resolve;

use kind::PathKind;
use options::ResolverOptions;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct Resolver {
    options: ResolverOptions,
    base_dir: Option<PathBuf>,
    cache_description_files: HashMap<PathBuf, serde_json::Value>,
}

type ResolverError = String;
type ResolverResult = Result<PathBuf, ResolverError>;

impl Resolver {
    pub fn with_base_dir(self, base_dir: &Path) -> Self {
        Self {
            base_dir: Some(base_dir.to_path_buf()),
            ..self
        }
    }

    pub fn use_base_dir(&mut self, base_dir: &Path) {
        self.base_dir = Some(base_dir.to_path_buf());
    }

    fn get_base_dir(&self) -> &PathBuf {
        self.base_dir
            .as_ref()
            .unwrap_or_else(|| panic!("base_dir is not set"))
    }

    pub(crate) fn get_description_file(&self, path: &PathBuf) -> Option<&serde_json::Value> {
        self.cache_description_files.get(path)
    }

    pub(crate) fn set_description_file(&mut self, path: &Path, value: serde_json::Value) {
        self.cache_description_files
            .insert(path.to_path_buf(), value);
    }

    pub fn resolve(&mut self, target: &str) -> ResolverResult {
        let target = &self.normalize_alias(target);
        let kind = self.get_path_kind(target);
        let may_be_dir = Resolver::may_be_dir(target);
        let base_dir = if matches!(kind, PathKind::Absolute) {
            PathBuf::from("/")
        } else {
            self.get_base_dir().clone()
        };

        match kind {
            PathKind::Empty => Err(ResolverError::from("Empty path")),
            PathKind::Relative | PathKind::Absolute => {
                if may_be_dir {
                    self.resolve_as_dir(&base_dir, target)
                } else {
                    self.resolve_as_file(&base_dir, target)
                        .or_else(|_| self.resolve_as_dir(&base_dir, target))
                        .and_then(|path| self.normalize_path(&path))
                }
            }
            PathKind::Internal => Ok(PathBuf::from(target)),
            PathKind::Normal => self
                .resolve_as_modules(&base_dir, target, may_be_dir)
                .and_then(|path| self.normalize_path(&path)),
        }
    }
}
