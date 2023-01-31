use crate::{normalize::NormalizePath, parse::Request};
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct Info {
    path: Box<Path>,
    request: Request,
}

impl<P: AsRef<Path>> From<P> for Info {
    fn from(path: P) -> Self {
        Self {
            path: path.as_ref().into(),
            request: Request::default(),
        }
    }
}

impl Info {
    #[must_use]
    pub fn new<P: AsRef<Path>>(path: P, request: Request) -> Self {
        Self {
            path: path.as_ref().into(),
            request,
        }
    }

    #[must_use]
    pub fn with_path<P: AsRef<Path>>(self, path: P) -> Self {
        Self {
            path: path.as_ref().into(),
            ..self
        }
    }

    #[must_use]
    pub fn with_request(self, request: Request) -> Self {
        Self { request, ..self }
    }

    #[must_use]
    pub fn with_target(self, target: &str) -> Self {
        let request = self.request.with_target(target);
        Self { request, ..self }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn request(&self) -> &Request {
        &self.request
    }

    #[must_use]
    pub fn to_resolved_path(&self) -> Cow<'_, Path> {
        if self.request.target().is_empty() || self.request.target() == "." {
            Cow::Borrowed(&self.path)
        } else {
            Cow::Owned(self.path.join(self.request.target()))
        }
    }

    #[must_use]
    pub fn normalize(mut self) -> Self {
        self.path = self.path.normalize().into();
        self
    }

    #[must_use]
    pub fn join(&self) -> PathBuf {
        let buf = format!(
            "{}{}{}",
            self.path.display(),
            self.request.query(),
            self.request.fragment(),
        );
        PathBuf::from(buf)
    }
}
