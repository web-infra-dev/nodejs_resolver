use super::Plugin;
use crate::{log, parse::Request, Context, Info, Resolver, State};

#[derive(Default)]
pub struct ParsePlugin;

impl Plugin for ParsePlugin {
    fn apply(&self, resolver: &Resolver, info: Info, context: &mut Context) -> State {
        let request = &info.request;
        let had_hash = !request.fragment.is_empty();
        let no_query = request.query.is_empty();
        let had_request = !info.request.target.is_empty();
        if no_query && had_hash && had_request {
            tracing::debug!("ParsePlugin works({})", log::depth(&context.depth));
            let directory = request.target.ends_with('/');
            let target = format!(
                "{}{}{}",
                request.target,
                if directory { "/" } else { "" },
                request.fragment
            );
            let kind = Resolver::get_target_kind(&target);
            let path = info.path.clone();
            let request = Request {
                target: target.into(),
                query: "".into(),
                fragment: "".into(),
                kind,
            };
            let info = Info::from(path, request);
            let state = resolver._resolve(info, context);
            if state.is_finished() {
                return state;
            }
            tracing::debug!("Leaving ParsePlugin({})", log::depth(&context.depth));
        }
        State::Resolving(info)
    }
}
