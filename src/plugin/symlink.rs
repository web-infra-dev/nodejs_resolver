use crate::{log::depth, Context, Info, ResolveResult, Resolver, State};

impl Resolver {
    pub async fn symlink_apply(&self, info: Info, context: &mut Context) -> State {
        debug_assert!(info.request().target().is_empty());

        if !self.options.symlinks {
            return State::Success(ResolveResult::Resource(info));
        }

        tracing::debug!("SymlinkPlugin works({})", depth(&context.depth));
        let state = self.resolve_symlink(info).await;
        tracing::debug!("Leaving SymlinkPlugin({})", depth(&context.depth));
        state
    }

    async fn resolve_symlink(&self, info: Info) -> State {
        let head = self.load_entry(info.normalized_path().as_ref());

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

            if let Some(link) = self.symlink(entry).await {
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

        let info = if let Some(symlink) = symlink {
            let mut path = symlink;
            let tail = entry_path.components().rev().take(index).collect::<Vec<_>>();
            for c in tail.into_iter().rev() {
                path.push(c);
            }
            head.init_real(path.clone().into_boxed_path());
            info.with_path(path)
        } else {
            stack.into_iter().for_each(|entry| entry.init_real(entry.path().into()));
            info
        };

        State::Success(ResolveResult::Resource(info))
    }
}
