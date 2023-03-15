use super::Plugin;
use crate::{
    description::PkgInfo, log::color, log::depth, normalize::NormalizePath, Context, Info,
    Resolver, State,
};

pub struct MainFieldPlugin<'a> {
    pkg_info: &'a PkgInfo,
}

impl<'a> MainFieldPlugin<'a> {
    pub fn new(pkg_info: &'a PkgInfo) -> Self {
        Self { pkg_info }
    }
}

impl<'a> Plugin for MainFieldPlugin<'a> {
    fn apply(&self, resolver: &Resolver, info: Info, context: &mut Context) -> State {
        let path = info.path().normalize();
        if !path.normalized_eq(&self.pkg_info.dir_path) {
            return State::Resolving(info);
        }
        let main_field_info = Info::new(&path, info.request().clone());
        for user_main_field in &resolver.options.main_fields {
            if let Some(main_field) = self
                .pkg_info
                .json
                .raw
                .get(user_main_field)
                .and_then(|value| value.as_str())
            {
                if main_field == "." || main_field == "./" {
                    // if it pointed to itself.
                    break;
                }
                tracing::debug!(
                    "MainField in '{}' works, using {} field({})",
                    color::blue(&format!(
                        "{}/package.json",
                        self.pkg_info.dir_path.display()
                    )),
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

                let state = resolver._resolve(main_field_info, context);
                if state.is_finished() {
                    return state;
                }
                tracing::debug!("Leaving MainField({})", depth(&context.depth));
            }
        }
        State::Resolving(info)
    }
}
