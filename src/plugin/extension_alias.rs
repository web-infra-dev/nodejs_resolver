use crate::{kind::PathKind, Context, Info, Resolver, State};

impl Resolver {
    pub async fn extension_alias_apply(
        &self,
        info: Info,
        extension: &str,
        alias_list: &Vec<String>,
        context: &mut Context,
    ) -> State {
        let request = info.request();
        let target = request.target();
        if matches!(request.kind(), PathKind::Normal)
            || target.is_empty()
            || !target.ends_with(extension)
        {
            State::Resolving(info)
        } else if !alias_list.is_empty() {
            for alias in alias_list {
                let target = &format!("{}{}", &target[0..target.len() - extension.len()], alias);
                let next = info.clone().with_target(target);
                let path = next.to_resolved_path().to_path_buf();
                let next = next.with_path(path).with_target("");
                let origin = context.fully_specified.get();
                context.fully_specified.set(true);
                let state = self._resolve(next, context).await;
                context.fully_specified.set(origin);
                if state.is_finished() {
                    return state;
                }
            }
            State::Failed(info)
        } else {
            State::Resolving(info)
        }
    }
}
