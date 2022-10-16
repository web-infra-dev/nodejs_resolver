use super::Plugin;
use crate::{
    context::Context,
    description::PkgInfo,
    map::{Field, ImportsField},
    Error, Info, PathKind, Resolver, State, MODULE,
};

pub struct ImportsFieldPlugin<'a> {
    pkg_info: &'a PkgInfo,
}

impl<'a> ImportsFieldPlugin<'a> {
    pub fn new(pkg_info: &'a PkgInfo) -> Self {
        Self { pkg_info }
    }

    fn check_target(resolver: &Resolver, info: Info, target: &str) -> State {
        let path = info.get_path();
        let is_file = match resolver.load_entry(&path) {
            Ok(entry) => entry.is_file(),
            Err(err) => return State::Error(err),
        };
        if is_file && ImportsField::check_target(&info.request.target) {
            State::Resolving(info)
        } else {
            State::Error(Error::UnexpectedValue(format!(
                "Package path {target} is not exported"
            )))
        }
    }
}

impl<'a> Plugin for ImportsFieldPlugin<'a> {
    fn apply(&self, resolver: &Resolver, info: Info, context: &mut Context) -> State {
        if !info.request.target.starts_with('#') {
            return State::Resolving(info);
        }

        let target = &info.request.target;
        let list = if let Some(root) = &self.pkg_info.json.imports_field_tree {
            match ImportsField::field_process(root, target, &resolver.options.condition_names) {
                Ok(list) => list,
                Err(err) => return State::Error(err),
            }
        } else {
            return State::Resolving(info);
        };

        if let Some(item) = list.first() {
            let request = resolver.parse(item);
            let is_normal_kind = matches!(request.kind, PathKind::Normal);
            let is_internal_kind = matches!(request.kind, PathKind::Internal);
            let info = Info::from(
                if is_normal_kind {
                    self.pkg_info.dir_path.join(MODULE)
                } else {
                    self.pkg_info.dir_path.to_path_buf()
                },
                request,
            );

            if is_normal_kind {
                let path = info.get_path();
                // TODO: should optimized
                let pkg_info = match resolver.load_entry(&path) {
                    Ok(entry) => entry.pkg_info.clone(),
                    Err(err) => return State::Error(err),
                };
                if let Some(ref pkg_info) = pkg_info {
                    if !pkg_info.dir_path.display().to_string().contains(MODULE) {
                        return State::Resolving(info);
                    }
                }

                let stats = resolver._resolve(info.clone(), context);
                if stats.is_finished() {
                    stats
                } else {
                    State::Resolving(info)
                }
            } else if is_internal_kind {
                self.apply(resolver, info, context)
            } else {
                ImportsFieldPlugin::check_target(resolver, info, target)
            }
        } else {
            State::Error(Error::UnexpectedValue(format!(
                "Package path {target} is not exported"
            )))
        }
    }
}
