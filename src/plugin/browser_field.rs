use std::path::{Path, PathBuf};

use path_absolutize::Absolutize;

use crate::{context::Context, description::DescriptionData, log::color, log::depth, AliasMap};
use crate::{Info, PathKind, ResolveResult, Resolver, State};

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

impl Resolver {
    pub async fn browser_field_apply(
        &self,
        info: Info,
        pkg_info: &DescriptionData,
        may_request_package_self: bool,
        context: &mut Context,
    ) -> State {
        if !self.options.browser_field {
            return State::Resolving(info);
        }

        for (alias_key, alias_target) in pkg_info.data().alias_fields() {
            let should_deal_alias = match matches!(info.request().kind(), PathKind::Normal)
                && !may_request_package_self
            {
                true => request_target_is_module_and_equal_alias_key(alias_key, &info),
                false => request_path_is_equal_alias_key_path(
                    &pkg_info.dir().as_ref().join(alias_key),
                    &info,
                    &self.options.extensions,
                ),
            };
            if !should_deal_alias {
                continue;
            }
            tracing::debug!(
                "BrowserFiled in '{}' works, trigger by '{}'({})",
                color::blue(&format!("{}/package.json", pkg_info.dir().as_ref().display())),
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

                    let alias_info = Info::from(pkg_info.dir().clone())
                        .with_request(info.request().clone())
                        .with_target(converted);
                    let fully_specified = context.fully_specified.get();
                    if fully_specified {
                        context.fully_specified.set(false);
                    }
                    let state = self._resolve(alias_info, context).await;
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
