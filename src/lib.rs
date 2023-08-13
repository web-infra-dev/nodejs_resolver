//! # `nodejs_resolver`
//!
//! ## How to use?
//!
//! ```rust, ignore
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
//! let builder = ResolverBuilder::new(yours_file_system);
//! let resolver = builder.build();
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

mod builder;
mod cache;
mod context;
mod description;
mod entry;
mod error;
mod fs;
mod fs_interface;
mod info;
mod kind;
mod log;
mod map;
mod options;
mod parse;
mod plugin;
mod resolve;
mod resource;
mod state;
mod tsconfig;
mod tsconfig_path;

pub use builder::ResolverBuilder;
pub use cache::Cache;
use context::Context;
pub use description::DescriptionData;
pub use error::Error;
pub use fs_interface::FileSystem;
use info::Info;
use kind::PathKind;
use log::{color, depth};
pub use options::{AliasMap, EnforceExtension, Options};
pub use resource::Resource;
use state::State;

#[derive(Debug)]
pub struct Resolver {
    pub options: Options,
    fs: Box<dyn FileSystem>,
    pub(crate) cache: std::sync::Arc<Cache>,
}

#[derive(Debug, Clone)]
pub enum ResolveResult<T: Clone> {
    Resource(T),
    Ignored,
}

pub type RResult<T> = Result<T, Error>;

impl Resolver {
    pub async fn resolve(
        &self,
        path: &std::path::Path,
        request: &str,
    ) -> RResult<ResolveResult<Resource>> {
        tracing::debug!(
            "{:-^30}\nTry to resolve '{}' in '{}'",
            color::green(&"[RESOLVER]"),
            color::cyan(&request),
            color::cyan(&path.display().to_string())
        );
        // let start = std::time::Instant::now();
        let parsed = Self::parse(request);
        let info = Info::new(path, parsed);
        let mut context =
            Context::new(self.options.fully_specified, self.options.resolve_to_context);
        let result = if let Some(tsconfig_location) = self.options.tsconfig.as_ref() {
            self._resolve_with_tsconfig(info, tsconfig_location, &mut context).await
        } else {
            self._resolve(info, &mut context).await
        };

        let result = if let State::Failed(info) = result {
            self.alias_apply(&self.options.fallback, info, &mut context).await
        } else {
            result
        };
        let result = if let State::Success(ResolveResult::Resource(info)) = result {
            self.symlink_apply(info, &mut context).await
        } else {
            result
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
            State::Success(ResolveResult::Ignored) => Ok(ResolveResult::Ignored),
            State::Success(ResolveResult::Resource(info)) => {
                let resource = Resource::new(info, self).await;
                Ok(ResolveResult::Resource(resource))
            }
            State::Error(err) => Err(err),
            State::Resolving(_) | State::Failed(_) => Err(Error::ResolveFailedTag),
        }
    }

    #[async_recursion::async_recursion]
    async fn _resolve(&self, info: Info, context: &mut Context) -> State {
        tracing::debug!(
            "Resolving '{request}' in '{path}'",
            request = color::cyan(&info.request().target()),
            path = color::cyan(&info.normalized_path().as_ref().display())
        );

        context.depth.increase();
        if context.depth.cmp(127).is_ge() {
            return State::Error(Error::Overflow);
        }
        let state = self.parse_apply(info, context).await;
        let State::Resolving(info) = state else {
            context.depth.decrease();
            return state;
        };
        let state = self.alias_apply(&self.options.alias, info, context).await;
        let State::Resolving(info) = state else {
            context.depth.decrease();
            return state;
        };
        let state = self.prefer_relative_apply(info, context).await;
        let State::Resolving(info) = state else {
            context.depth.decrease();
            return state;
        };
        let request = info.to_resolved_path();
        let entry = self.load_entry(&request);
        let pkg_info = match self.pkg_info(&entry).await {
            Ok(pkg_info) => pkg_info,
            Err(error) => return State::Error(error),
        };
        let state = if let Some(pkg_info) = pkg_info {
            self.imports_field_apply(info, pkg_info, context).await
        } else {
            State::Resolving(info)
        };
        let State::Resolving(info) = state else {
            context.depth.decrease();
            return state;
        };
        let state = if let Some(pkg_info) = pkg_info {
            self.browser_field_apply(info, pkg_info, false, context).await
        } else {
            State::Resolving(info)
        };
        let State::Resolving(info) = state else {
            context.depth.decrease();
            return state;
        };

        let state = if matches!(
            info.request().kind(),
            PathKind::AbsolutePosix | PathKind::AbsoluteWin | PathKind::Relative
        ) {
            let state = self.resolve_as_context(info, context).await;
            let State::Resolving(info) = state else {
                context.depth.decrease();
                return state;
            };
            let state = self.resolve_as_fully_specified(info, context).await;
            let State::Resolving(info) = state else {
                context.depth.decrease();
                return state;
            };
            let state = self.resolve_as_file(info, context).await;
            let State::Resolving(info) = state else {
                context.depth.decrease();
                return state;
            };
            self.resolve_as_dir(info, context).await
        } else {
            self.resolve_as_modules(info, context).await
        };

        context.depth.decrease();
        state
    }
}

#[cfg(debug_assertions)]
pub mod test_helper {
    #[must_use]
    pub fn p(paths: Vec<&str>) -> std::path::PathBuf {
        paths
            .iter()
            .fold(std::env::current_dir().unwrap().join("tests").join("fixtures"), |acc, path| {
                acc.join(path)
            })
    }

    #[must_use]
    pub fn vec_to_set(vec: Vec<&str>) -> std::collections::HashSet<String> {
        std::collections::HashSet::from_iter(vec.into_iter().map(|s| s.to_string()))
    }
}
