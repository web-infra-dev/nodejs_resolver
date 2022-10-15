use super::Plugin;
use crate::{description::PkgInfo, Info, Resolver, State};

pub struct MainFieldPlugin<'a> {
    pkg_info: &'a PkgInfo,
}

impl<'a> MainFieldPlugin<'a> {
    pub fn new(pkg_info: &'a PkgInfo) -> Self {
        Self { pkg_info }
    }
}

impl<'a> Plugin for MainFieldPlugin<'a> {
    fn apply(&self, resolver: &Resolver, info: Info) -> State {
        if !info.path.eq(&self.pkg_info.dir_path) {
            return State::Resolving(info);
        }

        let mut main_field_info = Info::from(info.path.to_owned(), info.request.clone());

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

                main_field_info = if main_field.starts_with("./") {
                    main_field_info.with_target(main_field)
                } else {
                    main_field_info.with_target(&format!("./{main_field}"))
                };

                let stats = resolver._resolve(main_field_info);
                if stats.is_success() {
                    return stats;
                } else {
                    main_field_info = stats.extract_info();
                }
            }
        }
        State::Resolving(info)
    }
}
