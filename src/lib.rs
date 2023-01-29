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
mod context;
mod description;
mod entry;
mod error;
mod fs;
mod info;
mod kind;
mod log;
mod map;
mod normalize;
mod options;
mod parse;
mod plugin;
mod resolve;
mod state;
mod tsconfig;
mod tsconfig_path;

use crate::normalize::NormalizePath;
pub use cache::Cache;
use context::Context;
pub use description::SideEffects;
use entry::Entry;
pub use error::Error;
pub use info::Info;
use kind::PathKind;
use log::*;
pub use options::{AliasMap, EnforceExtension, Options};
use plugin::{
    AliasPlugin, BrowserFieldPlugin, ImportsFieldPlugin, ParsePlugin, Plugin, PreferRelativePlugin,
};
use rustc_hash::FxHasher;
use state::State;
use std::{hash::BuildHasherDefault, path::Path, sync::Arc};
#[derive(Debug)]
pub struct Resolver {
    pub options: Options,
    pub(crate) cache: Arc<Cache>,
    // File entries keyed by normalized paths
    pub(crate) entries: dashmap::DashMap<Box<Path>, Arc<Entry>, BuildHasherDefault<FxHasher>>,
}

#[derive(Debug)]
pub enum ResolveResult {
    Info(Info),
    Ignored,
}

pub(crate) static MODULE: &str = "node_modules";

pub type RResult<T> = Result<T, Error>;

impl Resolver {
    pub fn new(options: Options) -> Self {
        log::enable_by_env();

        let cache = if let Some(external_cache) = options.external_cache.as_ref() {
            external_cache.clone()
        } else {
            Arc::new(Cache::default())
        };

        use options::EnforceExtension::*;
        let enforce_extension = match options.enforce_extension {
            Auto => {
                if options.extensions.iter().any(|ext| ext.is_empty()) {
                    Enabled
                } else {
                    Disabled
                }
            }
            _ => options.enforce_extension,
        };

        let options = Options {
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
        tracing::debug!(
            "{:-^30}\nTry to resolve '{}' in '{}'\n",
            color::green(&"[RESOLVER]"),
            color::cyan(&request),
            color::cyan(&path.display().to_string())
        );
        // let start = std::time::Instant::now();
        let parsed = self.parse(request);
        let info = Info::from(path.to_path_buf(), parsed);
        let mut context = Context::new();
        let result = if let Some(tsconfig_location) = self.options.tsconfig.as_ref() {
            self._resolve_with_tsconfig(info, tsconfig_location, &mut context)
        } else {
            self._resolve(info, &mut context)
        };
        // let duration = start.elapsed().as_millis();
        // println!("time cost: {:?} us", duration); // us
        // if duration > 10 {
        //     println!(
        //         "{:?}ms, path: {:?}, request: {:?}",
        //         duration,
        //         path.display(),
        //         request,
        //     );
        // }
        match result {
            State::Success(result) => self.normalize_result(result),
            State::Error(err) => Err(err),
            State::Resolving(_) | State::Failed(_) => Err(Error::ResolveFailedTag),
        }
    }

    #[tracing::instrument]
    fn _resolve(&self, info: Info, context: &mut Context) -> State {
        tracing::debug!(
            "Resolving '{request}' in '{path}'",
            request = color::cyan(&info.request.target),
            path = color::cyan(&info.path.display().to_string())
        );

        context.depth.increase();
        if context.depth.cmp(127).is_ge() {
            return State::Error(Error::Overflow);
        }

        let state = ParsePlugin::default()
            .apply(self, info, context)
            .then(|info| AliasPlugin::default().apply(self, info, context))
            .then(|info| PreferRelativePlugin::default().apply(self, info, context))
            .then(|info| {
                let request = info.get_path();
                let pkg_info = match self.load_entry(&request) {
                    Ok(entry) => entry.pkg_info.clone(),
                    Err(error) => return State::Error(error),
                };
                if let Some(pkg_info) = pkg_info {
                    ImportsFieldPlugin::new(&pkg_info)
                        .apply(self, info, context)
                        .then(|info| BrowserFieldPlugin::new(&pkg_info).apply(self, info, context))
                } else {
                    State::Resolving(info)
                }
            })
            .then(|info| {
                if matches!(
                    info.request.kind,
                    PathKind::AbsolutePosix | PathKind::AbsoluteWin | PathKind::Relative
                ) {
                    self.resolve_as_context(info)
                        .then(|info| self.resolve_as_file(info))
                        .then(|info| self.resolve_as_dir(info, context))
                } else {
                    self.resolve_as_modules(info, context)
                }
            });
        context.depth.decrease();
        state
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
