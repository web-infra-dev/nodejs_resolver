use crate::Resolver;

use super::Plugin;
use crate::{ResolverInfo, ResolverResult, ResolverStats};

#[derive(Default)]
pub struct ExtensionsPlugin;

impl Plugin for ExtensionsPlugin {
    fn apply(&self, resolver: &Resolver, info: ResolverInfo) -> ResolverStats {
        for extension in &resolver.options.extensions {
            let path = if info.request.target.is_empty() {
                Resolver::append_ext_for_path(&info.path, extension)
            } else {
                let str = if extension.is_empty() { "" } else { "." };
                info.path
                    .join(format!("{}{str}{extension}", info.request.target))
            };
            if path.is_file() {
                return ResolverStats::Success(ResolverResult::Info(
                    info.with_path(path).with_target(resolver, ""),
                ));
            }
        }
        ResolverStats::Resolving(info)
    }
}
