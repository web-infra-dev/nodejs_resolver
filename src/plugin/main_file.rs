use super::Plugin;
use crate::{log::color, log::depth, Context, Info, Resolver, State};

pub struct MainFilePlugin;

impl Plugin for MainFilePlugin {
    fn apply(&self, resolver: &Resolver, info: Info, context: &mut Context) -> State {
        let main_file_info = Info::from(info.path.clone(), info.request.clone());
        for main_file in &resolver.options.main_files {
            tracing::debug!(
                "MainFile works, it pointed to {}({})",
                color::blue(main_file),
                depth(&context.depth)
            );
            let main_file_info = main_file_info
                .clone()
                .with_target(&format!("./{main_file}"));
            let state = resolver._resolve(main_file_info, context);
            if state.is_finished() {
                return state;
            }
            tracing::debug!("Leaving MainFile({})", depth(&context.depth));
        }
        State::Resolving(info)
    }
}
