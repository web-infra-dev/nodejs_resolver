use std::{path::PathBuf, sync::Arc};

use crate::{description::DescriptionData, info::Info, Resolver};

#[derive(Debug, Clone)]
pub struct Resource {
    pub path: PathBuf,
    pub query: Option<String>,
    pub fragment: Option<String>,
    pub description: Option<Arc<DescriptionData>>,
}

impl Resource {
    pub(crate) fn new(info: Info, resolver: &Resolver) -> Self {
        let path = info.normalized_path().as_ref().to_path_buf();
        let query = info.request().query();
        let fragment = info.request().fragment();
        let description = resolver.load_entry(&path).pkg_info(resolver).unwrap().clone();
        Resource {
            path,
            query: (!query.is_empty()).then(|| query.into()),
            fragment: (!fragment.is_empty()).then(|| fragment.into()),
            description,
        }
    }

    pub fn join(&self) -> PathBuf {
        let mut buf = format!("{}", self.path.display());
        if let Some(query) = self.query.as_ref() {
            buf.push_str(query);
        }
        if let Some(fragment) = self.fragment.as_ref() {
            buf.push_str(fragment);
        }
        PathBuf::from(buf)
    }
}
