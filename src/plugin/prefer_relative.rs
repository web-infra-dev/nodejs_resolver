use crate::{kind::PathKind, log::depth, Context, Info, Resolver, State};

impl Resolver {
    pub async fn prefer_relative_apply(&self, info: Info, context: &mut Context) -> State {
        if matches!(info.request().kind(), PathKind::Relative) {
            return State::Resolving(info);
        }

        if self.options.prefer_relative {
            tracing::debug!("AliasPlugin works({})", depth(&context.depth));
            let target = format!("./{}", info.request().target());
            let info = info.clone().with_target(&target);
            let stats = self._resolve(info, context).await;
            if stats.is_finished() {
                return stats;
            }
            tracing::debug!("Leaving AliasPlugin({})", depth(&context.depth));
        }
        State::Resolving(info)
    }
}
