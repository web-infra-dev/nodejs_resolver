use std::borrow::Cow;
use std::path::{Path, PathBuf};

use crate::description::DescriptionData;
use crate::{info::NormalizedPath, kind::PathKind, log::color, Context, EnforceExtension, Info};
use crate::{ResolveResult, Resolver, State};

impl Resolver {
    async fn resolve_file_with_ext(&self, mut path: PathBuf, info: Info) -> State {
        // let v = unsafe { &mut *(&mut path as *mut PathBuf as *mut Vec<u8>) };
        let mut filename = path.file_name().unwrap().to_string_lossy().to_string();

        for ext in &self.options.extensions {
            filename.push_str(ext);
            path.set_file_name(&filename);
            let entry = self.load_entry(&path);
            if self.is_file(&entry).await {
                return State::Success(ResolveResult::Resource(
                    info.with_path(path).with_target(""),
                ));
            }
            for _ in 0..ext.len() {
                filename.pop();
            }
            path.set_file_name(&filename);
        }
        tracing::debug!(
            "'{}[{}]' is not a file",
            color::red(&path.display()),
            color::red(&self.options.extensions.join("|"))
        );
        State::Resolving(info)
    }

    pub(crate) async fn resolve_as_context(&self, info: Info, context: &Context) -> State {
        if !context.resolve_to_context.get() {
            return State::Resolving(info);
        }
        let path = info.to_resolved_path();
        tracing::debug!("Attempting to load '{}' as a context", color::blue(&path.display()));
        if self.is_dir(&self.load_entry(&path)).await {
            State::Success(ResolveResult::Resource(Info::new(path, Default::default())))
        } else {
            State::Failed(info)
        }
    }

    pub(crate) async fn resolve_as_fully_specified(
        &self,
        info: Info,
        context: &mut Context,
    ) -> State {
        let fully_specified = context.fully_specified.get();
        if !fully_specified {
            return State::Resolving(info);
        }
        let path = info.to_resolved_path();
        let request = info.request();
        let target = request.target();
        if self.is_file(&self.load_entry(&path)).await {
            let path = path.to_path_buf();
            State::Success(ResolveResult::Resource(info.with_path(path).with_target("")))
        } else if matches!(
            request.kind(),
            PathKind::AbsolutePosix | PathKind::AbsoluteWin | PathKind::Relative
        ) || split_slash_from_request(target).is_some()
        {
            State::Failed(info)
        } else {
            let dir = path.to_path_buf();
            let info = info.with_path(dir).with_target(".");
            context.fully_specified.set(false);
            let state = self._resolve(info.clone(), context).await;
            context.fully_specified.set(true);
            if state.is_finished() { state } else { State::Failed(info) }
        }
    }

    pub(crate) async fn resolve_as_file(&self, info: Info, context: &mut Context) -> State {
        if info.request().is_directory() {
            return State::Resolving(info);
        }

        let mut s = State::Resolving(info);
        for (extension, alias_list) in &self.options.extension_alias {
            if let State::Resolving(info) = s {
                s = self.extension_alias_apply(info.clone(), extension, alias_list, context).await;
            }
        }

        let State::Resolving(info) = s else {
            return s;
        };

        let path = info.to_resolved_path().to_path_buf();
        tracing::debug!("Attempting to load '{}' as a file", color::blue(&path.display()));
        if matches!(self.options.enforce_extension, EnforceExtension::Enabled) {
            self.resolve_file_with_ext(path, info).await
        } else if self.is_file(&self.load_entry(&path)).await {
            State::Success(ResolveResult::Resource(info.with_path(path).with_target("")))
        } else {
            self.resolve_file_with_ext(path, info).await
        }
    }

    pub(crate) async fn resolve_as_dir(&self, info: Info, context: &mut Context) -> State {
        let dir = info.to_resolved_path();
        let entry = self.load_entry(&dir);
        if !self.is_dir(&entry).await {
            return State::Failed(info);
        }
        let pkg_info = match self.pkg_info(&entry).await {
            Ok(pkg_info) => pkg_info,
            Err(err) => return State::Error(err),
        };
        let state = if let Some(pkg_info) = pkg_info {
            self.main_field_apply(info, pkg_info, context).await
        } else {
            State::Resolving(info)
        };
        if let State::Resolving(info) = state {
            self.main_file_apply(info, context).await
        } else {
            state
        }
    }

    pub(crate) async fn resolve_as_modules(&self, info: Info, context: &mut Context) -> State {
        let original_dir = info.normalized_path();
        for module in &self.options.modules {
            let node_modules_path = Path::new(module);
            let (node_modules_path, need_find_up) = if node_modules_path.is_absolute() {
                (Cow::Borrowed(node_modules_path), false)
            } else {
                (Cow::Owned(original_dir.as_ref().join(module)), true)
            };
            let state = self
                ._resolve_as_modules(info.clone(), original_dir, &node_modules_path, context)
                .await;
            let State::Resolving(info) = state else {
                return state;
            };
            let state = if !need_find_up {
                State::Resolving(info)
            } else if let Some(parent_dir) = original_dir.as_ref().parent() {
                self._resolve(info.with_path(parent_dir), context).await
            } else {
                State::Resolving(info)
            };
            if state.is_finished() {
                return state;
            }
        }
        State::Failed(info)
    }

    async fn _resolve_as_modules(
        &self,
        info: Info,
        original_dir: &NormalizedPath,
        node_modules_path: &Path,
        context: &mut Context,
    ) -> State {
        let entry = self.load_entry(node_modules_path);
        let pkg_info = match self.pkg_info(&entry).await {
            Ok(pkg_info) => pkg_info.as_ref(),
            Err(err) => return State::Error(err),
        };
        let state = if self.is_dir(&entry).await {
            // is there had `node_modules` folder?
            let state = self.resolve_node_modules(info, node_modules_path, context).await;
            let State::Resolving(info) = state else {
                return state;
            };
            let is_resolve_self = pkg_info.map_or(false, |pkg_info| {
                let request_module_name = get_module_name_from_request(info.request().target());
                is_resolve_self(pkg_info, request_module_name)
            });
            if is_resolve_self {
                let pkg_info = pkg_info.unwrap();
                self.exports_field_apply(pkg_info, info, context).await
            } else {
                State::Resolving(info)
            }
        } else if pkg_info.map_or(false, |pkg_info| pkg_info.dir().eq(original_dir)) {
            // is `info.path` on the same level as package.json
            let request_module_name = get_module_name_from_request(info.request().target());
            if is_resolve_self(pkg_info.unwrap(), request_module_name) {
                let pkg_info = pkg_info.unwrap();
                self.exports_field_apply(pkg_info, info, context).await
            } else {
                State::Resolving(info)
            }
        } else {
            State::Resolving(info)
        };

        state
    }

    async fn resolve_node_modules(
        &self,
        info: Info,
        node_modules_path: &Path,
        context: &mut Context,
    ) -> State {
        let original_dir = info.normalized_path();
        let request_module_name = get_module_name_from_request(info.request().target());
        let module_path = node_modules_path.join(request_module_name);
        let entry = self.load_entry(&module_path);
        let module_info = Info::new(node_modules_path, info.request().clone());
        if !self.is_dir(&entry).await {
            let state = self.resolve_as_file(module_info, context).await;
            if state.is_finished() { state } else { State::Resolving(info) }
        } else {
            let pkg_info = match self.pkg_info(&entry).await {
                Ok(pkg_info) => pkg_info,
                Err(err) => return State::Error(err),
            };
            let state = if let Some(pkg_info) = pkg_info {
                let out_node_modules = pkg_info.dir().eq(original_dir);
                let state = if !out_node_modules || is_resolve_self(pkg_info, request_module_name) {
                    self.exports_field_apply(pkg_info, module_info, context).await
                } else {
                    State::Resolving(module_info)
                };
                let State::Resolving(info) = state else {
                    return state;
                };
                let state = self.imports_field_apply(info, pkg_info, context).await;
                let State::Resolving(info) = state else {
                    return state;
                };
                let state = self.main_field_apply(info, pkg_info, context).await;
                let State::Resolving(info) = state else {
                    return state;
                };
                self.browser_field_apply(info, pkg_info, true, context).await
            } else {
                State::Resolving(module_info)
            };
            let State::Resolving(info) = state else {
                return state;
            };
            let state = self.resolve_as_context(info, context).await;
            let State::Resolving(info) = state else {
                return state;
            };
            let state = self.resolve_as_fully_specified(info, context).await;
            let State::Resolving(info) = state else {
                return state;
            };
            let state = self.resolve_as_file(info, context).await;
            let State::Resolving(info) = state else {
                return state;
            };
            let state = self.resolve_as_dir(info, context).await;
            match state {
                State::Failed(info) => State::Resolving(info),
                _ => state,
            }
        }
    }
}

fn is_resolve_self(pkg_info: &DescriptionData, request_module_name: &str) -> bool {
    pkg_info.data().name().map(|pkg_name| request_module_name == pkg_name).map_or(false, |ans| ans)
}

/// split the index from `[module-name]/[path]`
pub(crate) fn split_slash_from_request(target: &str) -> Option<usize> {
    let has_namespace_scope = target.starts_with('@');
    let chars = target.chars().enumerate();
    let slash_index_list: Vec<usize> =
        chars.filter_map(|(index, char)| if '/' == char { Some(index) } else { None }).collect();
    if has_namespace_scope { slash_index_list.get(1) } else { slash_index_list.first() }.copied()
}

fn get_module_name_from_request(target: &str) -> &str {
    split_slash_from_request(target).map_or(target, |index| &target[0..index])
}

pub(crate) fn get_path_from_request(target: &str) -> Option<Cow<str>> {
    split_slash_from_request(target).map(|index| Cow::Borrowed(&target[index..]))
}

#[cfg(test)]
mod test {
    use super::{get_module_name_from_request, get_path_from_request, split_slash_from_request};

    #[test]
    fn test_split_slash_from_request() {
        assert_eq!(split_slash_from_request("a"), None);
        assert_eq!(split_slash_from_request("a/b"), Some(1));
        assert_eq!(split_slash_from_request("@a"), None);
        assert_eq!(split_slash_from_request("@a/b"), None);
        assert_eq!(split_slash_from_request("@a/b/c"), Some(4));
    }

    #[test]
    fn test_get_module_name_from_request() {
        assert_eq!(get_module_name_from_request("a"), "a");
        assert_eq!(get_module_name_from_request("a/b"), "a");
        assert_eq!(get_module_name_from_request("@a"), "@a");
        assert_eq!(get_module_name_from_request("@a/b"), "@a/b");
        assert_eq!(get_module_name_from_request("@a/b/c"), "@a/b");
    }

    #[test]
    fn test_get_path_from_request() {
        assert_eq!(get_path_from_request("a"), None);
        assert_eq!(get_path_from_request("a/b"), Some("/b".into()));
        assert_eq!(get_path_from_request("@a"), None);
        assert_eq!(get_path_from_request("@a/b"), None);
        assert_eq!(get_path_from_request("@a/b/c"), Some("/c".into()));
    }
}
