use super::Plugin;
use crate::{Context, Info, Resolver, State};

#[derive(Default)]
pub struct PreferRelativePlugin;

impl Plugin for PreferRelativePlugin {
    fn apply(&self, resolver: &Resolver, info: Info, context: &mut Context) -> State {
        if info.request.target.starts_with("../") || info.request.target.starts_with("./") {
            return State::Resolving(info);
        }

        if resolver.options.prefer_relative {
            let target = format!("./{}", info.request.target);
            let info = Info::from(
                info.path.to_owned(),
                info.request.clone().with_target(&target),
            );
            let stats = resolver._resolve(info, context);
            if stats.is_finished() {
                return stats;
            }
        }
        State::Resolving(info)
    }
}
