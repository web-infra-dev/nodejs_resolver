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
//! let resolver = Resolver::new(Default::default());
//!
//! resolver.resolve(&cwd.join("./src"), "foo");
//! // -> ResolveResult::Info(ResolveInfo {
//! //    path: PathBuf::from("<cwd>/node_modules/foo/index.js")
//! //    request: Request {
//! //       target: "",
//! //       fragment: "",
//! //       query: ""
//! //    }
//! //  })
//! //
//!
//! resolver.resolve(&cwd.join("./src"), "./foo");
//! // -> ResolveResult::Info(ResolveInfo {
//! //    path: PathBuf::from("<cwd>/src/foo.js")
//! //    request: Request {
//! //       target: "",
//! //       fragment: "",
//! //       query: ""
//! //    }
//! //  })
//! //
//! ```
//!

mod cache;
mod description;
mod entry;
mod error;
mod fs;
mod kind;
mod map;
mod normalize;
mod options;
mod parse;
mod plugin;
mod resolve;
mod tsconfig;
mod tsconfig_path;

pub use cache::ResolverCache;
use dashmap::DashMap;
pub use description::SideEffects;
use entry::Entry;
pub use error::*;
use kind::PathKind;
pub use options::{AliasMap, ResolverOptions};
use parse::Request;
use plugin::{AliasFieldPlugin, AliasPlugin, ImportsFieldPlugin, Plugin, PreferRelativePlugin};

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Debug)]
pub struct Resolver {
    pub options: ResolverOptions,
    pub(crate) cache: Arc<ResolverCache>,
    pub(crate) entries: DashMap<PathBuf, Arc<Entry>>,
}

#[derive(Debug, Clone)]
pub struct ResolveInfo {
    pub path: PathBuf,
    pub request: Request,
}

impl ResolveInfo {
    pub fn from(path: PathBuf, request: Request) -> Self {
        Self { path, request }
    }

    pub fn get_path(&self) -> PathBuf {
        if self.request.target.is_empty() || self.request.target == "." {
            self.path.to_path_buf()
        } else {
            self.path.join(&*self.request.target)
        }
    }

    pub fn with_path(self, path: PathBuf) -> Self {
        Self { path, ..self }
    }

    pub fn with_target(self, target: &str) -> Self {
        let request = self.request.with_target(target);
        Self { request, ..self }
    }

    pub fn join(&self) -> PathBuf {
        let buf = format!(
            "{}{}{}",
            self.path.display(),
            self.request.query,
            self.request.fragment,
        );
        PathBuf::from(buf)
    }
}

#[derive(Debug)]
pub enum ResolveResult {
    Info(ResolveInfo),
    Ignored,
}

#[derive(Debug)]
pub(crate) enum ResolverStats {
    Success(ResolveResult),
    Resolving(ResolveInfo),
    Error((ResolverError, ResolveInfo)),
}

impl ResolverStats {
    pub fn and_then<F: FnOnce(ResolveInfo) -> ResolverStats>(self, op: F) -> ResolverStats {
        match self {
            ResolverStats::Resolving(info) => op(info),
            _ => self,
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, ResolverStats::Success(_))
    }

    pub fn extract_info(self) -> ResolveInfo {
        match self {
            ResolverStats::Resolving(info) => info,
            ResolverStats::Error((_, info)) => info,
            _ => unreachable!(),
        }
    }
}

pub(crate) static MODULE: &str = "node_modules";

pub type RResult<T> = Result<T, ResolverError>;

impl Resolver {
    pub fn new(options: ResolverOptions) -> Self {
        let cache = if let Some(external_cache) = options.external_cache.as_ref() {
            external_cache.clone()
        } else {
            Arc::new(ResolverCache::default())
        };
        let enforce_extension = if options.enforce_extension.is_none() {
            Some(options.extensions.iter().any(|ext| ext.is_empty()))
        } else {
            options.enforce_extension
        };
        let options = ResolverOptions {
            enforce_extension,
            ..options
        };
        let entries = Default::default();
        Self {
            cache,
            options,
            entries,
        }
    }

    #[tracing::instrument]
    pub fn resolve(&self, path: &Path, request: &str) -> RResult<ResolveResult> {
        // let start = std::time::Instant::now();
        let info = ResolveInfo::from(path.to_path_buf(), self.parse(request));

        let result = if let Some(tsconfig_location) = self.options.tsconfig.as_ref() {
            self._resolve_with_tsconfig(info, tsconfig_location)
        } else {
            self._resolve(info)
        };
        // let duration = start.elapsed().as_micros();
        // println!("time cost: {:?} us", duration); // us
        // if duration > 5000 {
        //     println!(
        //         "{:?}us, path: {:?}, request: {:?}",
        //         duration,
        //         path.display(),
        //         request,
        //     );
        // }
        match result {
            ResolverStats::Success(result) => self.normalize_result(result),
            ResolverStats::Error((err, _)) => Err(err),
            ResolverStats::Resolving(_) => Err(ResolverError::ResolveFailedTag),
        }
    }

    #[tracing::instrument]
    fn _resolve(&self, info: ResolveInfo) -> ResolverStats {
        AliasPlugin::default()
            .apply(self, info)
            .and_then(|info| PreferRelativePlugin::default().apply(self, info))
            .and_then(|info| {
                let request = if info.request.kind.eq(&PathKind::Normal) {
                    info.path.join(MODULE).join(&*info.request.target)
                } else {
                    info.get_path()
                };
                let pkg_info = match self.load_entry(&request) {
                    Ok(entry) => entry.pkg_info.clone(),
                    Err(error) => return ResolverStats::Error((error, info)),
                };
                if let Some(pkg_info) = pkg_info {
                    ImportsFieldPlugin::new(&pkg_info)
                        .apply(self, info)
                        .and_then(|info| AliasFieldPlugin::new(&pkg_info).apply(self, info))
                } else {
                    ResolverStats::Resolving(info)
                }
            })
            .and_then(|info| {
                if matches!(
                    info.request.kind,
                    PathKind::AbsolutePosix | PathKind::AbsoluteWin | PathKind::Relative
                ) {
                    self.resolve_as_file(info)
                        .and_then(|info| self.resolve_as_dir(info))
                } else {
                    self.resolve_as_modules(info)
                }
            })
    }
}

#[cfg(debug_assertions)]
pub mod test_helper {
    pub fn p(paths: Vec<&str>) -> std::path::PathBuf {
        paths.iter().fold(
            std::env::current_dir()
                .unwrap()
                .join("tests")
                .join("fixtures"),
            |acc, path| acc.join(path),
        )
    }

    pub fn vec_to_set(vec: Vec<&str>) -> std::collections::HashSet<String> {
        std::collections::HashSet::from_iter(vec.into_iter().map(|s| s.to_string()))
    }
}
