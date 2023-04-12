use super::Plugin;
use crate::{description::DescriptionData, log::color, log::depth, Context, Info, Resolver, State};

pub struct MainFieldPlugin<'a> {
    pkg_info: &'a DescriptionData,
}

impl<'a> MainFieldPlugin<'a> {
    pub fn new(pkg_info: &'a DescriptionData) -> Self {
        Self { pkg_info }
    }
}

impl<'a> Plugin for MainFieldPlugin<'a> {
    fn apply(&self, resolver: &Resolver, info: Info, context: &mut Context) -> State {
        let resolved = info.to_resolved_path();
        if !self.pkg_info.dir().as_ref().eq(&*resolved) {
            return State::Resolving(info);
        }
        let main_field_info = info.clone().with_path(resolved).with_target(".");

        for user_main_field in &resolver.options.main_fields {
            if let Some(main_field) = self
                .pkg_info
                .data()
                .raw()
                .get(user_main_field)
                .and_then(|value| value.as_str())
            {
                if main_field == "." || main_field == "./" {
                    // if it pointed to itself.
                    break;
                }
                tracing::debug!(
                    "MainField in '{}' works, using {} field({})",
                    color::blue(&format!("{:?}/package.json", self.pkg_info.dir().as_ref())),
                    color::blue(user_main_field),
                    depth(&context.depth)
                );

                let main_field_info = if main_field.starts_with("./") {
                    main_field_info.clone().with_target(main_field)
                } else {
                    main_field_info
                        .clone()
                        .with_target(&format!("./{main_field}"))
                };

                let fully_specified = context.fully_specified.get();
                if fully_specified {
                    context.fully_specified.set(false);
                }
                let state = resolver._resolve(main_field_info, context);
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
