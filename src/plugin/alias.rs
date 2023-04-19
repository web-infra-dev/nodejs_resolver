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
                            let request = Resolver::parse(&normalized_target);
                            let alias_info = info.clone().with_request(request);
                            let fully_specified = context.fully_specified.get();
                            if fully_specified {
                                context.fully_specified.set(false);
                            }
                            let state = resolver._resolve(alias_info, context);
                            if fully_specified {
                                context.fully_specified.set(true);
                            }
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
