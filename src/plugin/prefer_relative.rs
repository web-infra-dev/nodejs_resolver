use super::Plugin;
use crate::{log::depth, Context, Info, Resolver, State};

#[derive(Default)]
pub struct PreferRelativePlugin;

impl Plugin for PreferRelativePlugin {
    fn apply(&self, resolver: &Resolver, info: Info, context: &mut Context) -> State {
        if info.request.target().starts_with("../") || info.request.target().starts_with("./") {
            return State::Resolving(info);
        }

        if resolver.options.prefer_relative {
            tracing::debug!("AliasPlugin works({})", depth(&context.depth));
            let target = format!("./{}", info.request.target());
            let info = Info::from(info.path.clone(), info.request.clone().with_target(&target));
            let stats = resolver._resolve(info, context);
            if stats.is_finished() {
                return stats;
            }
            tracing::debug!("Leaving AliasPlugin({})", depth(&context.depth));
        }
        State::Resolving(info)
    }
}
