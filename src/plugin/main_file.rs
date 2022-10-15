use super::Plugin;
use crate::{Info, Resolver, State};

pub struct MainFilePlugin;

impl Plugin for MainFilePlugin {
    fn apply(&self, resolver: &Resolver, info: Info) -> State {
        let mut main_file_info = Info::from(info.path.to_owned(), info.request.clone());
        for main_file in &resolver.options.main_files {
            main_file_info = main_file_info.with_target(&format!("./{main_file}"));
            let stats = resolver._resolve(main_file_info);
            if stats.is_success() {
                return stats;
            } else {
                main_file_info = stats.extract_info();
            }
        }
        State::Resolving(info)
    }
}
