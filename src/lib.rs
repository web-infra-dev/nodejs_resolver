//! # nodejs_resolver
//!
//! ## How to use?
//!
//! ```rust
//! // |-- node_modules
//! // |---- foo
//! // |------ index.js
//! // | src
//! // |-- foo.ts
//! // |-- foo.js
//! // | tests
//!
//! use nodejs_resolver::Resolver;
//!
//! let cwd = std::env::current_dir().unwrap();
//! let mut resolver = Resolver::default()
//!                      .with_base_dir(&cwd.join("./src"));
//!
//! resolver.resolve("foo");
//! // -> Ok(<cwd>/node_modules/foo/index.js)
//!
//! resolver.resolve("./foo");
//! // -> Ok(<cwd>/src/foo.js)
//! ```
//!

mod description;
mod kind;
mod map;
mod normalize;
mod options;
mod parse;
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
        self._resolve(self.get_base_dir(), target.to_owned())
    }

    // pub fn resolve_from(&mut self, base_dir: &Path, target: &str) -> ResolverResult {
    //     self.resolve_inner(base_dir, target.to_owned())
    // }

    fn _resolve(&self, base_dir: &Path, target: String) -> ResolverResult {
        let normalized_target = &if let Some(target_after_alias) = self.normalize_alias(target) {
            target_after_alias
        } else {
            return Ok(None);
        };

        let part = Self::parse(normalized_target);
        let request = &part.request;
        let kind = Self::get_path_kind(request);
        let dir = match kind {
            PathKind::Empty => return Err(ResolverError::from("empty path")),
            PathKind::BuildInModule => return Ok(Some(PathBuf::from(request))),
            PathKind::AbsolutePosix | PathKind::AbsoluteWin => PathBuf::from("/"),
            _ => base_dir.to_path_buf(),
        };
        let description_file_info = self.load_description_file(&dir.join(request))?;
        let (base_dir, target) =
            match self.get_real_target(&dir, request, &kind, &description_file_info) {
                Some((dir, target)) => (dir, target),
                None => return Ok(None),
            };

        (if matches!(
            Self::get_path_kind(&target),
            PathKind::AbsolutePosix | PathKind::AbsoluteWin | PathKind::Relative
        ) {
            self.resolve_as_file(&base_dir, &target)
                .or_else(|_| self.resolve_as_dir(&base_dir, &target))
        } else {
            self.resolve_as_modules(&base_dir, &target)
        })
        .and_then(|path| self.normalize_path(path, &part))
    }

    // fn cache(&mut self) {
    //     if let Some(info) = description_file_info {
    //         self.cache_dir_info(&original_base_dir, &info.abs_dir_path);
    //         self.cache_description_file_info(info);
    //     }
    // }
}
