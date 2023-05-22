mod alias;
mod browser_field;
mod exports_field;
mod extension_alias;
mod imports_field;
mod main_field;
mod main_file;
mod parse;
mod prefer_relative;
mod symlink;

use crate::{context::Context, Info, Resolver, State};

pub use alias::AliasPlugin;
pub use browser_field::BrowserFieldPlugin;
pub use exports_field::ExportsFieldPlugin;
pub use extension_alias::ExtensionAliasPlugin;
pub use imports_field::ImportsFieldPlugin;
pub use main_field::MainFieldPlugin;
pub use main_file::MainFilePlugin;
pub use parse::ParsePlugin;
pub use prefer_relative::PreferRelativePlugin;
pub use symlink::SymlinkPlugin;

pub(crate) trait Plugin {
    fn apply(&self, resolver: &Resolver, info: Info, context: &mut Context) -> State;
}
