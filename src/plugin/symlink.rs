use crate::{log::depth, Context, Info, ResolveResult, Resolver, State};
use std::path::PathBuf;

#[derive(Default)]
pub struct SymlinkPlugin;

impl SymlinkPlugin {
    pub fn apply(resolver: &Resolver, info: Info, context: &mut Context) -> State {
        debug_assert!(info.request().target().is_empty());

        if !resolver.options.symlinks {
            return State::Success(ResolveResult::Resource(info));
        }

        tracing::debug!("SymlinkPlugin works({})", depth(&context.depth));
        let state = resolve_symlink(resolver, info, context);
        tracing::debug!("Leaving SymlinkPlugin({})", depth(&context.depth));
        state
    }
}

fn resolve_symlink(resolver: &Resolver, info: Info, _context: &mut Context) -> State {
    let head = resolver.load_entry(info.normalized_path().as_ref());

    let entry_path = head.path();
    let mut entry = head.as_ref();
    let mut index = 0;
    let mut symlink = None;
    let mut stack = vec![];

    loop {
        if let Some(real) = entry.real() {
            symlink = Some(real.to_path_buf());
            break;
        }

        if let Some(link) = entry.symlink() {
            symlink = Some(link.to_path_buf());
            break;
        }

        stack.push(entry);

        if let Some(e) = entry.parent() {
            index += 1;
            entry = e;
        } else {
            break;
        }
    }

    let path = if let Some(symlink) = symlink {
        let mut path = symlink;
        let tail = entry_path
            .components()
            .rev()
            .take(index)
            .collect::<Vec<_>>();
        for c in tail.into_iter().rev() {
            path.push(c);
        }
        head.init_real(path.clone().into_boxed_path());
        path
    } else {
        stack
            .into_iter()
            .for_each(|entry| entry.init_real(entry.path().into()));
        let mut path = PathBuf::default();
        for c in entry_path.components() {
            path.push(c);
        }
        path
    };
    let info = info.with_path(path);
    State::Success(ResolveResult::Resource(info))
}
