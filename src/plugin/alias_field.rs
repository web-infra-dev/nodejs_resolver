use crate::{
    context::Context, description::PkgInfo, AliasMap, Info, PathKind, Plugin, ResolveResult,
    Resolver, State,
};
use std::path::PathBuf;

pub struct AliasFieldPlugin<'a> {
    pkg_info: &'a PkgInfo,
}

impl<'a> AliasFieldPlugin<'a> {
    pub fn new(pkg_info: &'a PkgInfo) -> Self {
        Self { pkg_info }
    }

    fn request_target_is_module_and_equal_alias_key(alias_key: &String, info: &Info) -> bool {
        info.request.target.eq(alias_key)
    }

    fn request_path_is_equal_alias_key_path(
        alias_path: &PathBuf,
        info: &Info,
        extensions: &[String],
    ) -> bool {
        let request_path = info.get_path();
        alias_path.eq(&request_path)
            || extensions.iter().any(|ext| {
                let path_with_extension = Resolver::append_ext_for_path(&request_path, ext);
                alias_path.eq(&path_with_extension)
            })
    }
}

impl<'a> Plugin for AliasFieldPlugin<'a> {
    fn apply(&self, resolver: &Resolver, info: Info, context: &mut Context) -> State {
        if !resolver.options.browser_field {
            return State::Resolving(info);
        }
        for (alias_key, alias_target) in &self.pkg_info.json.alias_fields {
            let should_deal_alias = match matches!(info.request.kind, PathKind::Normal) {
                true => Self::request_target_is_module_and_equal_alias_key(alias_key, &info),
                false => Self::request_path_is_equal_alias_key_path(
                    &self.pkg_info.dir_path.join(alias_key),
                    &info,
                    &resolver.options.extensions,
                ),
            };
            if !should_deal_alias {
                continue;
            }
            match alias_target {
                AliasMap::Target(converted) => {
                    if alias_key == converted {
                        // pointed itself in `browser` field:
                        // {
                        //  "recursive": "recursive"
                        // }
                        return State::Resolving(info);
                    }
                    let alias_info = Info::from(
                        self.pkg_info.dir_path.to_path_buf(),
                        info.request.clone().with_target(converted),
                    );
                    let state = resolver._resolve(alias_info, context);
                    if state.is_finished() {
                        return state;
                    }
                }
                AliasMap::Ignored => return State::Success(ResolveResult::Ignored),
            };
        }
        State::Resolving(info)
    }
}
