use crate::{
    description::PkgInfo,
    log::color,
    log::depth,
    map::{ExportsField, Field},
    resolve::get_path_from_request,
    Context, Error, Info, Resolver, State,
};

use super::Plugin;

pub struct ExportsFieldPlugin<'a> {
    pkg_info: &'a PkgInfo,
}

impl<'a> ExportsFieldPlugin<'a> {
    pub fn new(pkg_info: &'a PkgInfo) -> Self {
        Self { pkg_info }
    }
}

impl<'a> Plugin for ExportsFieldPlugin<'a> {
    fn apply(&self, resolver: &Resolver, info: Info, context: &mut Context) -> State {
        // info.path should end with `node_modules`.
        let target = info.request().target();

        let list = if let Some(root) = &self.pkg_info.json.exports_field_tree {
            let query = info.request().query();
            let fragment = info.request().fragment();
            let request_path = get_path_from_request(target);

            let target = match request_path {
                Some(target) => format!(".{target}"),
                None => {
                    let path = info.path().join(target);
                    let is_exist = match resolver.load_entry(&path) {
                        Ok(entry) => entry.exists(),
                        Err(err) => return State::Error(err),
                    };
                    if is_exist
                        || self
                            .pkg_info
                            .json
                            .name
                            .as_ref()
                            .map_or(false, |name| name.eq(target))
                    {
                        ".".to_string()
                    } else {
                        return State::Failed(info);
                    }
                }
            };
            let remaining_target = if !query.is_empty() || !fragment.is_empty() {
                let target = if target == "." {
                    String::from("./")
                } else {
                    target
                };
                format!("{target}{query}{fragment}")
            } else {
                target
            };

            match ExportsField::field_process(
                root,
                &remaining_target,
                &resolver.options.condition_names,
            ) {
                Ok(list) => list,
                Err(err) => return State::Error(err),
            }
        } else {
            return State::Resolving(info);
        };

        for item in list {
            tracing::debug!(
                "ExportsField in '{}' works, trigger by '{}', mapped to '{}'({})",
                color::blue(&format!(
                    "{}/package.json",
                    self.pkg_info.dir_path.display()
                )),
                color::blue(&target),
                color::blue(&item),
                depth(&context.depth)
            );
            let request = Resolver::parse(&item);
            let info = Info::new(self.pkg_info.dir_path.clone(), request);
            if !ExportsField::check_target(info.request().target()) {
                continue;
            }
            let state = resolver._resolve(info, context);
            if state.is_finished() {
                return state;
            }
        }

        State::Error(Error::UnexpectedValue(format!(
            "Package path {target} is not exported in {}/package.json",
            self.pkg_info.dir_path.display()
        )))
        // TODO: `info.abs_dir_path.as_os_str().to_str().unwrap(),` has abs_path
    }
}
