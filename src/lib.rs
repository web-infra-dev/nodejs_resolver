//! # `nodejs_resolver`
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

pub use cache::Cache;
use context::Context;
pub use description::SideEffects;
pub use error::Error;
pub use info::Info;
use kind::PathKind;
use log::{color, depth};
use options::EnforceExtension::{Auto, Disabled, Enabled};
pub use options::{AliasMap, EnforceExtension, Options};
use plugin::{
    AliasPlugin, BrowserFieldPlugin, ImportsFieldPlugin, ParsePlugin, Plugin, PreferRelativePlugin,
    SymlinkPlugin,
};
use state::State;
use std::{path::Path, sync::Arc};

#[derive(Debug)]
pub struct Resolver {
    pub options: Options,
    pub(crate) cache: Arc<Cache>,
}

#[derive(Debug)]
pub enum ResolveResult {
    Info(Info),
    Ignored,
}

pub type RResult<T> = Result<T, Error>;

impl Resolver {
    #[must_use]
    pub fn new(options: Options) -> Self {
        log::enable_by_env();

        let cache = if let Some(external_cache) = options.external_cache.as_ref() {
            external_cache.clone()
        } else {
            Arc::new(Cache::default())
        };

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
        Self { options, cache }
    }

    #[tracing::instrument]
    pub fn resolve(&self, path: &Path, request: &str) -> RResult<ResolveResult> {
        tracing::debug!(
            "{:-^30}\nTry to resolve '{}' in '{}'",
            color::green(&"[RESOLVER]"),
            color::cyan(&request),
            color::cyan(&path.display().to_string())
        );
        // let start = std::time::Instant::now();
        let parsed = Self::parse(request);
        let info = Info::new(path, parsed);
        let mut context = Context::new();
        let result = if let Some(tsconfig_location) = self.options.tsconfig.as_ref() {
            self._resolve_with_tsconfig(info, tsconfig_location, &mut context)
        } else {
            self._resolve(info, &mut context)
        };

        let result = result.map_failed(|info| {
            type FallbackPlugin<'a> = AliasPlugin<'a>;
            FallbackPlugin::new(&self.options.fallback).apply(self, info, &mut context)
        });
        let result =
            result.map_success(|info| SymlinkPlugin::default().apply(self, info, &mut context));

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
            State::Success(result) => Ok(result),
            State::Error(err) => Err(err),
            State::Resolving(_) | State::Failed(_) => Err(Error::ResolveFailedTag),
        }
    }

    #[tracing::instrument]
    fn _resolve(&self, info: Info, context: &mut Context) -> State {
        tracing::debug!(
            "Resolving '{request}' in '{path}'",
            request = color::cyan(&info.request().target()),
            path = color::cyan(&info.path().display())
        );

        context.depth.increase();
        if context.depth.cmp(127).is_ge() {
            return State::Error(Error::Overflow);
        }

        let state = ParsePlugin::default()
            .apply(self, info, context)
            .then(|info| AliasPlugin::new(&self.options.alias).apply(self, info, context))
            .then(|info| PreferRelativePlugin::default().apply(self, info, context))
            .then(|info| {
                let request = info.to_resolved_path();
                let entry = self.load_entry(&request);
                let pkg_info = match entry.pkg_info(self) {
                    Ok(pkg_info) => pkg_info,
                    Err(error) => return State::Error(error),
                };
                if let Some(pkg_info) = pkg_info {
                    ImportsFieldPlugin::new(pkg_info)
                        .apply(self, info, context)
                        .then(|info| BrowserFieldPlugin::new(pkg_info).apply(self, info, context))
                } else {
                    State::Resolving(info)
                }
            })
            .then(|info| {
                if matches!(
                    info.request().kind(),
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
    #[must_use]
    pub fn p(paths: Vec<&str>) -> std::path::PathBuf {
        paths.iter().fold(
            std::env::current_dir()
                .unwrap()
                .join("tests")
                .join("fixtures"),
            |acc, path| acc.join(path),
        )
    }

    #[must_use]
    pub fn vec_to_set(vec: Vec<&str>) -> std::collections::HashSet<String> {
        std::collections::HashSet::from_iter(vec.into_iter().map(|s| s.to_string()))
    }
}
