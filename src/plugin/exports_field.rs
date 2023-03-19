use crate::{
    description::DescriptionData,
    log::color,
    log::depth,
    map::{ExportsField, Field},
    resolve::get_path_from_request,
    Context, Error, Info, Resolver, State,
};

use super::Plugin;

pub struct ExportsFieldPlugin<'a> {
    pkg_info: &'a DescriptionData,
}

impl<'a> ExportsFieldPlugin<'a> {
    pub fn new(pkg_info: &'a DescriptionData) -> Self {
        Self { pkg_info }
    }
}

impl<'a> Plugin for ExportsFieldPlugin<'a> {
    fn apply(&self, resolver: &Resolver, info: Info, context: &mut Context) -> State {
        let root = match self.pkg_info.data().exports_tree() {
            Ok(Some(exports_tree)) => exports_tree,
            Ok(None) => return State::Resolving(info),
            Err(error) => match error {
                Error::UnexpectedValue(value) => {
                    return State::Error(Error::UnexpectedValue(value.to_string()))
                }
                _ => unreachable!(),
            },
        };
        // info.path should end with `node_modules`.
        let target = info.request().target();
        let query = info.request().query();
        let fragment = info.request().fragment();
        let request_path = get_path_from_request(target);

        let normalized_target = match request_path {
            Some(target) => format!(".{target}"),
            None => {
                let path = info.normalized_path().as_ref().join(target);
                if resolver.load_entry(&path).exists()
                    || self
                        .pkg_info
                        .data()
                        .name()
                        .map_or(false, |name| target == name)
                {
                    ".".to_string()
                } else {
                    return State::Failed(info);
                }
            }
        };

        let remaining_target = if !query.is_empty() || !fragment.is_empty() {
            let normalized_target = if normalized_target == "." {
                String::from("./")
            } else {
                normalized_target
            };
            format!("{normalized_target}{query}{fragment}")
        } else {
            normalized_target
        };

        let list = match ExportsField::field_process(
            root,
            &remaining_target,
            &resolver.options.condition_names,
        ) {
            Ok(list) => list,
            Err(err) => return State::Error(err),
        };

        for item in list {
            tracing::debug!(
                "ExportsField in '{}' works, trigger by '{}', mapped to '{}'({})",
                color::blue(&format!(
                    "{}/package.json",
                    self.pkg_info.dir().as_ref().display()
                )),
                color::blue(&target),
                color::blue(&item),
                depth(&context.depth)
            );
            let request = Resolver::parse(&item);
            let info = Info::from(self.pkg_info.dir().clone()).with_request(request);
            if let Err(msg) = ExportsField::check_target(info.request().target()) {
                let msg = format!("{msg} in {:?}/package.json", &self.pkg_info.dir());
                return State::Error(Error::UnexpectedValue(msg));
            }
            let state = resolver._resolve(info, context);
            if state.is_finished() {
                return state;
            }
        }

        State::Error(Error::UnexpectedValue(format!(
            "Package path {target} is not exported in {}/package.json",
            self.pkg_info.dir().as_ref().display()
        )))
        // TODO: `info.abs_dir_path.as_os_str().to_str().unwrap(),` has abs_path
    }
}
