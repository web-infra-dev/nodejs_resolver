use crate::{description::DescriptionData, log::color, log::depth, Context, Info, Resolver, State};

impl Resolver {
    pub async fn main_field_apply(
        &self,
        info: Info,
        pkg_info: &DescriptionData,
        context: &mut Context,
    ) -> State {
        let resolved = info.to_resolved_path();
        if !pkg_info.dir().as_ref().eq(&*resolved) {
            return State::Resolving(info);
        }
        let main_field_info = info.clone().with_path(resolved).with_target(".");

        for user_main_field in &self.options.main_fields {
            if let Some(main_field) =
                pkg_info.data().raw().get(user_main_field).and_then(|value| value.as_str())
            {
                if main_field == "." || main_field == "./" {
                    // if it pointed to itself.
                    break;
                }
                tracing::debug!(
                    "MainField in '{}' works, using {} field({})",
                    color::blue(&format!("{:?}/package.json", pkg_info.dir().as_ref())),
                    color::blue(user_main_field),
                    depth(&context.depth)
                );

                let main_field_info = if main_field.starts_with("./") {
                    main_field_info.clone().with_target(main_field)
                } else {
                    main_field_info.clone().with_target(&format!("./{main_field}"))
                };

                let fully_specified = context.fully_specified.get();
                if fully_specified {
                    context.fully_specified.set(false);
                }
                let state = self._resolve(main_field_info, context).await;
                if fully_specified {
                    context.fully_specified.set(true);
                }
                if state.is_finished() {
                    return state;
                }
                tracing::debug!("Leaving MainField({})", depth(&context.depth));
            }
        }
        State::Resolving(info)
    }
}
