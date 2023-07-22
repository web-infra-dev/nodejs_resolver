use std::path::{Path, PathBuf};

use path_absolutize::Absolutize;

use crate::{context::Context, description::DescriptionData, log::color, log::depth, AliasMap};
use crate::{Info, PathKind, Plugin, ResolveResult, Resolver, State};

pub struct BrowserFieldPlugin<'a> {
    pkg_info: &'a DescriptionData,
    may_request_package_self: bool,
}

impl<'a> BrowserFieldPlugin<'a> {
    pub fn new(pkg_info: &'a DescriptionData, may_request_package_self: bool) -> Self {
        Self { pkg_info, may_request_package_self }
    }

    fn request_target_is_module_and_equal_alias_key(alias_key: &String, info: &Info) -> bool {
        info.request().target().eq(alias_key)
    }

    fn request_path_is_equal_alias_key_path(
        alias_path: &Path,
        info: &Info,
        extensions: &[String],
    ) -> bool {
        let alias_path = alias_path.absolutize_from(Path::new("")).unwrap();
        let request_path = info.to_resolved_path();
        let mut request_path = request_path.absolutize_from(Path::new("")).unwrap().to_path_buf();
        let v = unsafe { &mut *(&mut request_path as *mut PathBuf as *mut Vec<u8>) };

        alias_path.eq(&request_path)
            || extensions.iter().any(|ext| {
                v.extend_from_slice(ext.as_bytes());
                let result = alias_path.eq(&request_path);
                unsafe {
                    v.set_len(v.len() - ext.len());
                }
                result
            })
    }
}

impl<'a> Plugin for BrowserFieldPlugin<'a> {
    fn apply(&self, resolver: &Resolver, info: Info, context: &mut Context) -> State {
        if !resolver.options.browser_field {
            return State::Resolving(info);
        }

        for (alias_key, alias_target) in self.pkg_info.data().alias_fields() {
            let should_deal_alias = match matches!(info.request().kind(), PathKind::Normal)
                && !self.may_request_package_self
            {
                true => Self::request_target_is_module_and_equal_alias_key(alias_key, &info),
                false => Self::request_path_is_equal_alias_key_path(
                    &self.pkg_info.dir().as_ref().join(alias_key),
                    &info,
                    &resolver.options.extensions,
                ),
            };
            if !should_deal_alias {
                continue;
            }
            tracing::debug!(
                "BrowserFiled in '{}' works, trigger by '{}'({})",
                color::blue(&format!("{}/package.json", self.pkg_info.dir().as_ref().display())),
                color::blue(alias_key),
                depth(&context.depth)
            );
            match alias_target {
                AliasMap::Target(converted) => {
                    if alias_key == converted {
                        // pointed itself in `browser` field:
                        // {
                        //  "recursive": "recursive"
                        // }
                        return State::Resolving(info);
                    }

                    let alias_info = Info::from(self.pkg_info.dir().clone())
                        .with_request(info.request().clone())
                        .with_target(converted);
                    let fully_specified = context.fully_specified.get();
                    if fully_specified {
                        context.fully_specified.set(false);
                    }
                    let state = resolver._resolve(alias_info, context);
                    if fully_specified {
                        context.fully_specified.set(true);
                    }
                    if state.is_finished() {
                        return state;
                    }
                    tracing::debug!("Leaving BrowserFiled({})", depth(&context.depth));
                }
                AliasMap::Ignored => return State::Success(ResolveResult::Ignored),
            };
        }
        State::Resolving(info)
    }
}
