mod alias;
mod alias_field;
mod exports_field;
mod extensions;
mod imports_field;
mod main_field;
mod main_file;
mod prefer_relative;

use crate::{ResolveInfo, Resolver, ResolverStats};

pub use alias::AliasPlugin;
pub use alias_field::AliasFieldPlugin;
pub use exports_field::ExportsFieldPlugin;
pub use extensions::ExtensionsPlugin;
pub use imports_field::ImportsFieldPlugin;
pub use main_field::MainFieldPlugin;
pub use main_file::MainFilePlugin;
pub use prefer_relative::PreferRelativePlugin;

pub(crate) trait Plugin {
    fn apply(&self, resolver: &Resolver, info: ResolveInfo) -> ResolverStats;
}
