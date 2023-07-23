use crate::map::{ExportsField, Field};
use crate::{description::DescriptionData, log::color, log::depth, resolve::get_path_from_request};
use crate::{Context, Error, Info, Resolver, State};

impl Resolver {
    pub async fn exports_field_apply(
        &self,
        pkg_info: &DescriptionData,
        info: Info,
        context: &mut Context,
    ) -> State {
        let request = info.request();
        let target = request.target();

        for field in &self.options.exports_field {
            let root = match pkg_info.data().get_filed(field) {
                Some(exports_tree) => exports_tree,
                None => continue,
            };

            if request.is_directory() {
                return State::Error(Error::UnexpectedValue(format!(
                    "Resolving to directories is not possible with the exports field (request was {}/ in {})",
                    target,
                    info.normalized_path().as_ref().display()
                )));
            }

            let query = request.query();
            let fragment = request.fragment();
            let request_path = get_path_from_request(target);

            let normalized_target = match request_path {
                Some(target) => format!(".{target}"),
                None => {
                    let path = info.normalized_path().as_ref().join(target);
                    if self.exists(&self.load_entry(&path)).await
                        || pkg_info.data().name().map_or(false, |name| target == name)
                    {
                        ".".to_string()
                    } else {
                        return State::Failed(info);
                    }
                }
            };

            let remaining_target = if !query.is_empty() || !fragment.is_empty() {
                let normalized_target =
                    if normalized_target == "." { String::from("./") } else { normalized_target };
                format!("{normalized_target}{query}{fragment}")
            } else {
                normalized_target
            };

            let list = match ExportsField::field_process(
                root,
                &remaining_target,
                &self.options.condition_names,
            ) {
                Ok(list) => list,
                Err(err) => return State::Error(err),
            };

            if list.is_empty() {
                return State::Error(Error::UnexpectedValue(format!(
                    "Package path {target} is not exported in {}/package.json",
                    pkg_info.dir().as_ref().display()
                )));
            }

            for item in list {
                tracing::debug!(
                    "ExportsField in '{}' works, trigger by '{}', mapped to '{}'({})",
                    color::blue(&format!("{}/package.json", pkg_info.dir().as_ref().display())),
                    color::blue(&target),
                    color::blue(&item),
                    depth(&context.depth)
                );
                if !item.starts_with("./") {
                    return State::Error(Error::UnexpectedValue(format!(
                        "Invalid \"{item}\" defined in {}/package.json, target must start with  \"./\"",
                        pkg_info.dir().as_ref().display()
                    )));
                }
                let request = Resolver::parse(&item);
                let info = Info::from(pkg_info.dir().clone()).with_request(request);
                if let Err(msg) = ExportsField::check_target(info.request().target()) {
                    let msg = format!("{msg} in {:?}/package.json", &pkg_info.dir());
                    return State::Error(Error::UnexpectedValue(msg));
                }
                let state = self._resolve(info, context).await;
                if state.is_finished() {
                    return state;
                }
            }

            return State::Failed(info);
        }

        State::Resolving(info)
    }
}
