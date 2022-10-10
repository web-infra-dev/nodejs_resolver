use crate::{description::PkgInfo, Resolver};

use super::{Plugin, ResolveInfo, ResolverStats};

pub struct MainFieldPlugin<'a> {
    pkg_info: &'a Option<PkgInfo>,
}

impl<'a> MainFieldPlugin<'a> {
    pub fn new(pkg_info: &'a Option<PkgInfo>) -> Self {
        Self { pkg_info }
    }
}

impl<'a> Plugin for MainFieldPlugin<'a> {
    fn apply(&self, resolver: &Resolver, info: ResolveInfo) -> ResolverStats {
        if let Some(pkg_info) = self.pkg_info {
            if !info.path.eq(&pkg_info.abs_dir_path) {
                return ResolverStats::Resolving(info);
            }

            let mut main_field_info = ResolveInfo::from(info.path.to_owned(), info.request.clone());

            for user_main_field in &resolver.options.main_fields {
                if let Some(main_field) = pkg_info
                    .inner
                    .raw
                    .get(user_main_field)
                    .and_then(|value| value.as_str())
                {
                    if main_field == "." || main_field == "./" {
                        // if it pointed to itself.
                        break;
                    }

                    main_field_info = if main_field.starts_with("./") {
                        main_field_info.with_target(main_field)
                    } else {
                        main_field_info.with_target(&format!("./{main_field}"))
                    };

                    let stats = resolver._resolve(main_field_info);
                    if stats.is_success() {
                        return stats;
                    } else {
                        main_field_info = stats.extract_info();
                    }
                }
            }
        }
        ResolverStats::Resolving(info)
    }
}
