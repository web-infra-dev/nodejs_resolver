use super::Plugin;
use crate::{kind::PathKind, Context, Info, Resolver, State};

pub struct ExtensionAliasPlugin<'a> {
    extension: &'a str,
    alias_list: &'a Vec<String>,
}

impl<'a> ExtensionAliasPlugin<'a> {
    pub fn new(extension: &'a str, alias_list: &'a Vec<String>) -> Self {
        Self { extension, alias_list }
    }
}

impl<'a> Plugin for ExtensionAliasPlugin<'a> {
    fn apply(&self, resolver: &Resolver, info: Info, context: &mut Context) -> State {
        let request = info.request();
        let target = request.target();
        if matches!(request.kind(), PathKind::Normal)
            || target.is_empty()
            || !target.ends_with(self.extension)
        {
            State::Resolving(info)
        } else if !self.alias_list.is_empty() {
            for alias in self.alias_list {
                let target =
                    &format!("{}{}", &target[0..target.len() - self.extension.len()], alias);
                let next = info.clone().with_target(target);
                let path = next.to_resolved_path().to_path_buf();
                let next = next.with_path(path).with_target("");
                let origin = context.fully_specified.get();
                context.fully_specified.set(true);
                let state = resolver._resolve(next, context);
                context.fully_specified.set(origin);
                if state.is_finished() {
                    return state;
                }
            }
            State::Failed(info)
        } else {
            State::Resolving(info)
        }
    }
}
