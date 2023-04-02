use super::Plugin;
use crate::{
    context::Context,
    description::DescriptionData,
    log::color,
    log::depth,
    map::{Field, ImportsField},
    Error, Info, PathKind, Resolver, State,
};

pub struct ImportsFieldPlugin<'a> {
    pkg_info: &'a DescriptionData,
}

impl<'a> ImportsFieldPlugin<'a> {
    pub fn new(pkg_info: &'a DescriptionData) -> Self {
        Self { pkg_info }
    }

    fn check_target(&self, resolver: &Resolver, info: Info) -> State {
        let path = info.to_resolved_path();
        if resolver.load_entry(&path).is_file() {
            if let Err(msg) = ImportsField::check_target(info.request().target()) {
                let msg = format!("{msg} in {:?}/package.json", &self.pkg_info.dir().as_ref());
                State::Error(Error::UnexpectedValue(msg))
            } else {
                State::Resolving(info)
            }
        } else {
            State::Error(Error::UnexpectedValue(format!(
                "Package path {} can't imported in {:?}",
                info.request().target(),
                info.normalized_path().as_ref()
            )))
        }
    }
}

impl<'a> Plugin for ImportsFieldPlugin<'a> {
    fn apply(&self, resolver: &Resolver, info: Info, context: &mut Context) -> State {
        if !info.request().target().starts_with('#') {
            return State::Resolving(info);
        }

        let root = match self.pkg_info.data().imports_tree() {
            Ok(Some(tree)) => tree,
            Ok(None) => return State::Resolving(info),
            Err(error) => match error {
                Error::UnexpectedValue(value) => {
                    return State::Error(Error::UnexpectedValue(value.to_string()))
                }
                _ => unreachable!(),
            },
        };

        let list = match ImportsField::field_process(
            root,
            info.request().target(),
            &resolver.options.condition_names,
        ) {
            Ok(list) => list,
            Err(err) => return State::Error(err),
        };

        if let Some(item) = list.first() {
            tracing::debug!(
                "ImportsField in '{}' works, trigger by '{}', mapped to '{}'({})",
                color::blue(&format!("{:?}/package.json", self.pkg_info.dir().as_ref())),
                color::blue(&info.request().target()),
                color::blue(&item),
                depth(&context.depth)
            );
            let request = Resolver::parse(item);
            let is_relative = !matches!(request.kind(), PathKind::Normal | PathKind::Internal);
            let info = Info::from(self.pkg_info.dir().clone()).with_request(request);
            if is_relative {
                self.check_target(resolver, info)
            } else {
                let fully_specified = context.fully_specified.get();
                if fully_specified {
                    context.fully_specified.set(false);
                }
                let state = resolver._resolve(info, context);
                if fully_specified {
                    context.fully_specified.set(true);
                }
                state
            }
        } else {
            State::Error(Error::UnexpectedValue(format!(
                "Package path {} can't imported in {:?}",
                info.request().target(),
                info.normalized_path().as_ref()
            )))
        }
    }
}
