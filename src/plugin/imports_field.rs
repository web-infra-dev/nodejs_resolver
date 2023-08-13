use crate::map::{Field, ImportsField};
use crate::{context::Context, description::DescriptionData, log::color, log::depth};
use crate::{Error, Info, PathKind, Resolver, State};

impl Resolver {
    async fn check_target(&self, info: Info, pkg_info: &DescriptionData) -> State {
        let path = info.to_resolved_path();
        if self.is_file(&self.load_entry(&path)).await {
            if let Err(msg) = ImportsField::check_target(info.request().target()) {
                let msg = format!("{msg} in {:?}/package.json", pkg_info.dir().as_ref());
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

    pub async fn imports_field_apply(
        &self,
        info: Info,
        pkg_info: &DescriptionData,
        context: &mut Context,
    ) -> State {
        if !info.request().target().starts_with('#') {
            return State::Resolving(info);
        }

        let root = match pkg_info.data().raw().get("imports") {
            Some(tree) => tree,
            None => return State::Resolving(info),
        };

        let list = match ImportsField::field_process(
            root,
            info.request().target(),
            &self.options.condition_names,
        ) {
            Ok(list) => list,
            Err(err) => return State::Error(err),
        };

        if let Some(item) = list.first() {
            tracing::debug!(
                "ImportsField in '{}' works, trigger by '{}', mapped to '{}'({})",
                color::blue(&format!("{:?}/package.json", pkg_info.dir().as_ref())),
                color::blue(&info.request().target()),
                color::blue(&item),
                depth(&context.depth)
            );
            let request = Resolver::parse(item);
            let is_relative = !matches!(request.kind(), PathKind::Normal | PathKind::Internal);
            let info = Info::from(pkg_info.dir().clone()).with_request(request);
            if is_relative {
                self.check_target(info, pkg_info).await
            } else {
                let fully_specified = context.fully_specified.get();
                if fully_specified {
                    context.fully_specified.set(false);
                }
                let state = self._resolve(info, context).await;
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
