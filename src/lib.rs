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
//! let mut resolver = Resolver::default();
//!
//! resolver.resolve(&cwd.join("./src"), "foo");
//! // -> ResolveResult::Path(PathBuf::from("<cwd>/node_modules/foo/index.js"))
//!
//! resolver.resolve(&cwd.join("./src"), "./foo");
//! // -> ResolveResult::Path(PathBuf::from("<cwd>/src/foo.js"))
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
use parse::Request;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct Resolver {
    options: ResolverOptions,
    cache_dir_info: HashMap<PathBuf, DirInfo>,
    cache_description_file_info: HashMap<PathBuf, DescriptionFileInfo>,
}

pub struct DirInfo {
    pub description_file_path: PathBuf,
}

// TODO: should remove `Clone`
#[derive(Clone, Debug)]
pub struct Stats {
    pub dir: PathBuf,
    pub request: Request,
}

impl Stats {
    pub fn from(dir: PathBuf, request: Request) -> Self {
        Self { dir, request }
    }

    pub fn get_path(&self) -> PathBuf {
        if self.request.target.is_empty() {
            self.dir.to_path_buf()
        } else {
            self.dir.join(&self.request.target)
        }
    }

    pub fn with_dir(self, dir: PathBuf) -> Self {
        Self { dir, ..self }
    }

    pub fn with_target(self, target: String) -> Self {
        Self {
            request: Request {
                target,
                ..self.request
            },
            ..self
        }
    }
}

type ResolverError = String;
type RResult<T> = Result<T, ResolverError>;
type ResolverStats = RResult<Option<Stats>>;

#[derive(Debug, PartialEq)]
pub enum ResolveResult {
    Path(PathBuf),
    Ignored,
}
type ResolverResult = RResult<ResolveResult>;

impl Resolver {
    pub fn resolve(&mut self, base_dir: &PathBuf, target: &str) -> ResolverResult {
        self._resolve(base_dir, target.to_string())
    }

    fn _resolve(&self, base_dir: &Path, target: String) -> ResolverResult {
        let normalized_target = &if let Some(target_after_alias) = self.normalize_alias(target) {
            target_after_alias
        } else {
            return Ok(ResolveResult::Ignored);
        };
        let stats = Stats::from(base_dir.to_path_buf(), Self::parse(normalized_target));
        let init_query = stats.request.query.clone();
        let init_fragment = stats.request.fragment.clone();

        let kind = Self::get_target_kind(&stats.request.target);
        let dir = match kind {
            PathKind::Empty => return Err(format!("Can't resolve '' in {}", base_dir.display())),
            PathKind::BuildInModule => {
                return Ok(ResolveResult::Path(PathBuf::from(&stats.request.target)))
            }
            PathKind::AbsolutePosix | PathKind::AbsoluteWin => PathBuf::from("/"),
            _ => base_dir.to_path_buf(),
        };
        let stats = stats.with_dir(dir);

        let description_file_info =
            self.load_description_file(&stats.dir.join(&stats.request.target))?;
        let stats = match self.get_real_target(stats, &kind, &description_file_info, false)? {
            Some(stats) => stats,
            None => return Ok(ResolveResult::Ignored),
        };

        if matches!(
            Self::get_target_kind(&stats.request.target),
            PathKind::AbsolutePosix | PathKind::AbsoluteWin | PathKind::Relative
        ) {
            self.resolve_as_file(&stats)
                .or_else(|_| match self.resolve_as_dir(stats, false)? {
                    Some(stats) => Ok(ResolveResult::Path(stats.get_path())),
                    None => Ok(ResolveResult::Ignored),
                })
        } else {
            match self.resolve_as_modules(stats)? {
                Some(stats) => Ok(ResolveResult::Path(stats.get_path())),
                None => Ok(ResolveResult::Ignored),
            }
        }
        .and_then(|result| self.normalize_path(result, &init_query, &init_fragment))
    }

    // fn cache(&mut self) {
    //     if let Some(info) = description_file_info {
    //         self.cache_dir_info(&original_base_dir, &info.abs_dir_path);
    //         self.cache_description_file_info(info);
    //     }
    // }
}
