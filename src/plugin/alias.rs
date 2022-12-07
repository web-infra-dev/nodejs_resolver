use super::Plugin;
use crate::{log, AliasMap, Context, Info, ResolveResult, Resolver, State};

#[derive(Default)]
pub struct AliasPlugin;

impl Plugin for AliasPlugin {
    fn apply(&self, resolver: &Resolver, info: Info, context: &mut Context) -> State {
        let inner_target = &info.request.target;
        for (from, to) in &resolver.options.alias {
            if inner_target == from || inner_target.starts_with(&format!("{}/", from)) {
                tracing::debug!(
                    "AliasPlugin works, triggered by '{from}'({})",
                    log::depth(&context.depth)
                );
                match to {
                    AliasMap::Target(to) => {
                        if inner_target.starts_with(to) {
                            // skip `target.starts_with(to)` to prevent infinite loop.
                            continue;
                        }
                        let normalized_target = inner_target.replacen(from, to, 1);
                        let alias_info = Info::from(
                            info.path.to_path_buf(),
                            info.request.clone().with_target(&normalized_target),
                        );
                        let state = resolver._resolve(alias_info, context);
                        if state.is_finished() {
                            return state;
                        }
                    }
                    AliasMap::Ignored => return State::Success(ResolveResult::Ignored),
                }
                tracing::debug!("Leaving AliasPlugin({})", log::depth(&context.depth));
            }
        }

        State::Resolving(info)
    }
}
