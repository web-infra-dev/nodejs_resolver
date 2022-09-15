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
mod utils;

pub use cache::ResolverCache;
pub use description::SideEffects;
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
    cache: Arc<ResolverCache>,
    fs: fs::FileSystem,
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
        // if a file changed in 3 seconds,
        // it will reread this file.
        let fs = fs::FileSystem::new(3);
        Self { fs, cache, options }
    }

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
        // let resolve_err_msg = Self::raise_resolve_failed_message(&info);
        AliasPlugin::default()
            .apply(self, info)
            .and_then(|info| PreferRelativePlugin::default().apply(self, info))
            .and_then(|info| {
                let request = if info.request.kind.eq(&PathKind::Normal) {
                    info.path.join(MODULE).join(&*info.request.target)
                } else {
                    info.get_path()
                };
                let pkg_info_wrap = match self.load_pkg_file(&request) {
                    Ok(pkg_info_wrap) => pkg_info_wrap,
                    Err(error) => return ResolverStats::Error((error, info)),
                };
                ImportsFieldPlugin::new(&pkg_info_wrap)
                    .apply(self, info)
                    .and_then(|info| AliasFieldPlugin::new(&pkg_info_wrap).apply(self, info))
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

#[cfg(test)]
mod test {

    use super::test_helper::p;
    use super::*;

    #[test]
    fn pkg_info_cache_test() {
        let fixture_path = p(vec![]);
        // show tracing tree
        tracing_span_tree::span_tree().aggregate(true).enable();
        let resolver = Resolver::new(ResolverOptions {
            ..Default::default()
        });
        assert!(resolver
            .resolve(&fixture_path, "./browser-module/lib/browser")
            .is_ok());

        let full_path = fixture_path.join("full").join("a");
        assert!(resolver.resolve(&full_path, "package3").is_ok());

        assert_eq!(resolver.cache.file_dir_to_pkg_info.len(), 2);

        assert_eq!(
            resolver
                .cache
                .file_dir_to_pkg_info
                .get(&p(vec!["browser-module"]))
                .unwrap()
                .as_ref()
                .unwrap()
                .abs_dir_path,
            p(vec!["browser-module"])
        );
        assert_eq!(
            resolver
                .cache
                .file_dir_to_pkg_info
                .get(&p(vec!["full", "a", "node_modules", "package3"]))
                .unwrap()
                .as_ref()
                .unwrap()
                .abs_dir_path,
            p(vec!["full", "a", "node_modules", "package3"])
        );

        // should hit `cache.file_dir_to_pkg_info`.
        let _ = resolver.resolve(&fixture_path, "./browser-module/lib/browser");
        let _ = resolver.resolve(&full_path, "package3");
    }

    #[test]
    fn shared_cache_test1() {
        let cache = Arc::new(ResolverCache::default());
        let fixture_path = p(vec![]);

        let resolver = Resolver::new(ResolverOptions {
            external_cache: Some(cache.clone()),
            ..Default::default()
        });
        let _ = resolver.resolve(&fixture_path, "./browser-module/lib/browser");
        assert_eq!(cache.file_dir_to_pkg_info.len(), 1);
        assert_eq!(resolver.cache.file_dir_to_pkg_info.len(), 1);

        let resolver = Resolver::new(ResolverOptions {
            external_cache: Some(cache.clone()),
            ..Default::default()
        });

        let full_path = p(vec!["full", "a"]);
        let _ = resolver.resolve(&full_path, "package3");
        assert_eq!(cache.file_dir_to_pkg_info.len(), 2);
        assert_eq!(resolver.cache.file_dir_to_pkg_info.len(), 2);
    }
}
