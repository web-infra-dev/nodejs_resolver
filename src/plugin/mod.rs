mod alias;
mod alias_field;
mod exports_field;
mod imports_field;
mod main_field;
mod main_file;
mod prefer_relative;

use crate::{Resolver, ResolverInfo, ResolverStats};

pub use alias::AliasPlugin;
pub use alias_field::AliasFieldPlugin;
pub use exports_field::ExportsFieldPlugin;
pub use imports_field::ImportsFieldPlugin;
pub use main_field::MainFieldPlugin;
pub use main_file::MainFilePlugin;
pub use prefer_relative::PreferRelativePlugin;

pub(crate) trait Plugin {
    fn apply(&self, resolver: &Resolver, info: ResolverInfo) -> ResolverStats;
}
