use super::Plugin;
use crate::{log::depth, options::Alias, AliasMap, Context, Info, ResolveResult, Resolver, State};

pub struct AliasPlugin<'a>(&'a Alias);

impl<'a> AliasPlugin<'a> {
    pub fn new(alias: &'a Alias) -> Self {
        Self(alias)
    }

    fn alias(&self) -> &Alias {
        self.0
    }
}

impl<'a> Plugin for AliasPlugin<'a> {
    fn apply(&self, resolver: &Resolver, info: Info, context: &mut Context) -> State {
        let inner_target = info.request().target();
        for (from, array) in self.alias() {
            if inner_target
                .strip_prefix(from)
                .into_iter()
                .next()
                .map_or(false, |c| c.is_empty() || c.starts_with('/'))
            {
                tracing::debug!(
                    "AliasPlugin works, triggered by '{from}'({})",
                    depth(&context.depth)
                );
                for to in array {
                    match to {
                        AliasMap::Target(to) => {
                            if inner_target.starts_with(to) {
                                // skip `target.starts_with(to)` to prevent infinite loop.
                                continue;
                            }
                            let normalized_target = inner_target.replacen(from, to, 1);
                            let alias_info = Info::new(
                                info.path(),
                                info.request().clone().with_target(&normalized_target),
                            );
                            let state = resolver._resolve(alias_info, context);
                            if state.is_finished() {
                                return state;
                            }
                        }
                        AliasMap::Ignored => return State::Success(ResolveResult::Ignored),
                    }
                }
                tracing::debug!("Leaving AliasPlugin({})", depth(&context.depth));
            }
        }

        State::Resolving(info)
    }
}
