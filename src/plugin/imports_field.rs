use super::Plugin;
use crate::{
    context::Context,
    description::PkgInfo,
    log::color,
    log::depth,
    map::{Field, ImportsField},
    Error, Info, PathKind, Resolver, State,
};

pub struct ImportsFieldPlugin<'a> {
    pkg_info: &'a PkgInfo,
}

impl<'a> ImportsFieldPlugin<'a> {
    pub fn new(pkg_info: &'a PkgInfo) -> Self {
        Self { pkg_info }
    }

    fn check_target(&self, resolver: &Resolver, info: Info) -> State {
        let path = info.to_resolved_path();
        let is_file = match resolver.load_entry(&path) {
            Ok(entry) => entry.is_file(),
            Err(err) => return State::Error(err),
        };
        if is_file {
            if let Err(msg) = ImportsField::check_target(info.request().target()) {
                let msg = format!("{msg} in {:?}/package.json", &self.pkg_info.dir_path);
                State::Error(Error::UnexpectedValue(msg))
            } else {
                State::Resolving(info)
            }
        } else {
            State::Error(Error::UnexpectedValue(format!(
                "Package path {} can't imported in {}",
                info.request().target(),
                info.path().display()
            )))
        }
    }
}

impl<'a> Plugin for ImportsFieldPlugin<'a> {
    fn apply(&self, resolver: &Resolver, info: Info, context: &mut Context) -> State {
        if !info.request().target().starts_with('#') {
            return State::Resolving(info);
        }

        let list = if let Some(root) = &self.pkg_info.json.imports_field_tree {
            match ImportsField::field_process(
                root,
                info.request().target(),
                &resolver.options.condition_names,
            ) {
                Ok(list) => list,
                Err(err) => return State::Error(err),
            }
        } else {
            return State::Resolving(info);
        };

        if let Some(item) = list.first() {
            tracing::debug!(
                "ImportsField in '{}' works, trigger by '{}', mapped to '{}'({})",
                color::blue(&format!(
                    "{}/package.json",
                    self.pkg_info.dir_path.display()
                )),
                color::blue(&info.request().target()),
                color::blue(&item),
                depth(&context.depth)
            );
            let request = Resolver::parse(item);
            let is_relative = !matches!(request.kind(), PathKind::Normal | PathKind::Internal);
            let info = Info::new(&self.pkg_info.dir_path, request);
            if is_relative {
                self.check_target(resolver, info)
            } else {
                resolver._resolve(info, context)
            }
        } else {
            State::Error(Error::UnexpectedValue(format!(
                "Package path {} can't imported in {}",
                info.request().target(),
                info.path().display()
            )))
        }
    }
}
