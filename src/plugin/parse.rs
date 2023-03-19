use super::Plugin;
use crate::{depth, Context, Info, Resolver, State};

#[derive(Default)]
pub struct ParsePlugin;

impl Plugin for ParsePlugin {
    fn apply(&self, resolver: &Resolver, info: Info, context: &mut Context) -> State {
        let request = info.request();
        let had_hash = !request.fragment().is_empty();
        let no_query = request.query().is_empty();
        let had_request = !info.request().target().is_empty();
        if no_query && had_hash && had_request {
            tracing::debug!("ParsePlugin works({})", depth(&context.depth));
            let target = format!(
                "{}{}{}",
                request.target(),
                if request.is_directory() { "/" } else { "" },
                request.fragment()
            );
            let info = Info::from(info.normalized_path().clone()).with_target(&target);
            let state = resolver._resolve(info, context);
            if state.is_finished() {
                return state;
            }
            tracing::debug!("Leaving ParsePlugin({})", depth(&context.depth));
        }
        State::Resolving(info)
    }
}
