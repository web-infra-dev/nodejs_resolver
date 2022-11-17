use crate::{
    description::PkgInfo,
    map::{ExportsField, Field},
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
        let target = &info.request.target;

        let list = if let Some(root) = &self.pkg_info.json.exports_field_tree {
            let query = &info.request.query;
            let fragment = &info.request.fragment;
            let chars: String = if target.starts_with('@') {
                let index = target.find('/').unwrap();
                &target[index + 1..]
            } else {
                target
            }
            .chars()
            .collect();

            let target = match chars.find('/').map(|index| &chars[index..]) {
                Some(target) => format!(".{target}"),
                None => {
                    let target = target.to_string();
                    if info.path.join(&target).exists() || self.pkg_info.json.name.eq(&Some(target))
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

        use crate::ResolveResult;
        for item in list {
            let request = resolver.parse(&item);
            let info = Info::from(self.pkg_info.dir_path.to_path_buf(), request);
            if !ExportsField::check_target(&info.request.target) {
                continue;
            }
            let result = match resolver._resolve(info, context) {
                State::Success(result) => result,
                _ => continue,
            };
            let info = match result {
                ResolveResult::Info(info) => info,
                ResolveResult::Ignored => return State::Success(ResolveResult::Ignored),
            };
            let path = info.get_path();
            let is_file = match resolver.load_entry(&path) {
                Ok(entry) => entry.is_file(),
                Err(err) => return State::Error(err),
            };
            if is_file {
                return State::Success(ResolveResult::Info(info));
            }
        }

        State::Error(Error::UnexpectedValue(format!(
            "Package path {target} is not exported {}",
            info.path.display()
        )))
        // TODO: `info.abs_dir_path.as_os_str().to_str().unwrap(),` has abs_path
    }
}
