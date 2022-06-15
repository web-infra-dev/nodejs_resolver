use crate::{Resolver, ResolverInfo};

pub static RAISE_RESOLVE_ERROR_TAG: &str = "T0";

impl Resolver {
    pub(super) fn raise_tag() -> String {
        RAISE_RESOLVE_ERROR_TAG.to_string()
    }

    pub(super) fn raise_resolve_failed_message(info: &ResolverInfo) -> String {
        format!(
            "Resolve '{}' failed in '{}'",
            info.request,
            info.path.display()
        )
    }
}
