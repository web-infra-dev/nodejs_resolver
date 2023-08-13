use crate::{log::color, log::depth, Context, Info, Resolver, State};

impl Resolver {
    pub async fn main_file_apply(&self, info: Info, context: &mut Context) -> State {
        let path = info.to_resolved_path();
        for main_file in &self.options.main_files {
            tracing::debug!(
                "MainFile works, it pointed to {}({})",
                color::blue(main_file),
                depth(&context.depth)
            );
            let main_file_info =
                info.clone().with_path(&path).with_target(&format!("./{main_file}"));
            let state = self._resolve(main_file_info, context).await;
            if state.is_finished() {
                return state;
            }
            tracing::debug!("Leaving MainFile({})", depth(&context.depth));
        }
        State::Resolving(info)
    }
}
