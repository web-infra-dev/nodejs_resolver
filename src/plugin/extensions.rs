use std::path::PathBuf;

use crate::Resolver;

use super::Plugin;
use crate::{ResolveInfo, ResolveResult, ResolverStats};

pub struct ExtensionsPlugin {
    path: PathBuf,
}

impl ExtensionsPlugin {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Plugin for ExtensionsPlugin {
    fn apply(&self, resolver: &Resolver, info: ResolveInfo) -> ResolverStats {
        for extension in &resolver.options.extensions {
            let path = Resolver::append_ext_for_path(&self.path, extension);
            if path.is_file() {
                return ResolverStats::Success(ResolveResult::Info(
                    info.with_path(path).with_target(""),
                ));
            }
        }
        ResolverStats::Resolving(info)
    }
}
