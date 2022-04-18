mod description;
mod kind;
mod normalize;
mod options;
mod resolve;

use description::DescriptionFileInfo;
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
    cache_dir_info: HashMap<PathBuf, DirInfo>,
    cache_description_file_info: HashMap<PathBuf, DescriptionFileInfo>,
}

pub struct DirInfo {
    pub description_file_path: PathBuf,
}

type ResolverError = String;
type RResult<T> = Result<T, ResolverError>;
type ResolverResult = RResult<Option<PathBuf>>;

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

    pub fn resolve(&mut self, target: &str) -> ResolverResult {
        let original_target = &if let Some(target_after_alias) = self.normalize_alias(target) {
            target_after_alias
        } else {
            return Ok(None);
        };

        let kind = self.get_path_kind(original_target);
        let original_base_dir = match &kind {
            PathKind::Empty => return Err(ResolverError::from("empty path")),
            PathKind::Internal => return Ok(Some(PathBuf::from(target))),
            PathKind::Absolute => PathBuf::from("/"),
            _ => self.get_base_dir().clone(),
        };

        let target_path = match kind {
            // TODO: use `self.options.modules`.
            PathKind::NormalModule => {
                original_base_dir.join(format!("node_modules/{}", original_target))
            }
            _ => original_base_dir.join(original_target),
        };
        let description_file_info = self.load_description_file(&target_path);
        let (base_dir, target) =
            match self.get_real_target(&original_base_dir, target, &kind, &description_file_info) {
                Some((dir, target)) => {
                    if let Some(target) = target {
                        (dir, target)
                    } else {
                        return Ok(None);
                    }
                }
                None => (original_base_dir.clone(), original_target.to_string()),
            };

        let result = if matches!(
            self.get_path_kind(&target),
            PathKind::Absolute | PathKind::Relative
        ) {
            self.resolve_as_file(&base_dir, &target)
                .or_else(|_| self.resolve_as_dir(&description_file_info, &base_dir, &target))
                .and_then(|path| self.normalize_path(path))
        } else {
            self.resolve_as_modules(&base_dir, &target, &description_file_info)
                .and_then(|path| self.normalize_path(path))
        };
        if let Some(info) = description_file_info {
            self.cache_dir_info(&original_base_dir, &info.abs_dir_path);
            self.cache_description_file_info(info);
        }
        result
    }
}
