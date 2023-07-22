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
            let only_module = from.ends_with('$');
            let from_to = from.len();
            let (hit, key) = if only_module {
                let sub = &from[0..from_to - 1];
                if inner_target.eq(sub) { (true, sub) } else { (false, sub) }
            } else {
                let hit = inner_target
                    .strip_prefix(from)
                    .into_iter()
                    .next()
                    .map_or(false, |c| c.is_empty() || c.starts_with('/'));
                (hit, from.as_str())
            };
            if hit {
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
                            let normalized_target = inner_target.replacen(key, to, 1);
                            let old_request = info.request();
                            let old_query = old_request.query();
                            let old_fragment = old_request.fragment();
                            let request = Resolver::parse(&normalized_target);
                            let request =
                                match (request.query().is_empty(), request.fragment().is_empty()) {
                                    (true, true) => {
                                        request.with_query(old_query).with_fragment(old_fragment)
                                    }
                                    (true, false) => request.with_query(old_query),
                                    (false, true) => request.with_fragment(old_fragment),
                                    (false, false) => request,
                                };
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
