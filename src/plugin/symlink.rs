use super::Plugin;
use crate::{log::depth, Context, Info, ResolveResult, Resolver, State};

#[derive(Default)]
pub struct SymlinkPlugin;

impl Plugin for SymlinkPlugin {
    fn apply(&self, resolver: &Resolver, info: Info, context: &mut Context) -> State {
        debug_assert!(info.request().target().is_empty());

        if !resolver.options.symlinks {
            let info = info.normalize();
            return State::Success(ResolveResult::Info(info));
        }

        tracing::debug!("SymlinkPlugin works({})", depth(&context.depth));
        let state = self.resolve_symlink(resolver, info, context);
        tracing::debug!("Leaving SymlinkPlugin({})", depth(&context.depth));
        state
    }
}

impl SymlinkPlugin {
    fn resolve_symlink(&self, resolver: &Resolver, info: Info, _context: &mut Context) -> State {
        let entry = resolver.load_entry(info.path());
        let entry_path = entry.path();
        let mut entry = entry.as_ref();
        let mut index = 0;
        let mut symlink = None;

        loop {
            if let Some(link) = entry.symlink() {
                symlink = Some(link.to_path_buf());
                break;
            }
            if let Some(e) = entry.parent() {
                index += 1;
                entry = e;
            } else {
                break;
            }
        }

        let info = if let Some(symlink) = symlink {
            let mut path = symlink;
            let tail = entry_path
                .components()
                .rev()
                .take(index)
                .collect::<Vec<_>>();
            for c in tail.into_iter().rev() {
                path.push(c);
            }
            info.with_path(path)
        } else {
            info.normalize()
        };

        State::Success(ResolveResult::Info(info))
    }
}
