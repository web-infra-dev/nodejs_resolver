use crate::parse::Request;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Info {
    pub path: PathBuf,
    pub request: Request,
}

impl Info {
    #[must_use]
    pub fn from(path: PathBuf, request: Request) -> Self {
        Self { path, request }
    }

    #[must_use]
    pub fn get_path(&self) -> PathBuf {
        if self.request.target.is_empty() || self.request.target == "." {
            self.path.clone()
        } else {
            self.path.join(&*self.request.target)
        }
    }

    #[must_use]
    pub fn with_path(self, path: PathBuf) -> Self {
        Self { path, ..self }
    }

    #[must_use]
    pub fn with_target(self, target: &str) -> Self {
        let request = self.request.with_target(target);
        Self { request, ..self }
    }

    #[must_use]
    pub fn join(&self) -> PathBuf {
        let buf = format!(
            "{}{}{}",
            self.path.display(),
            self.request.query,
            self.request.fragment,
        );
        PathBuf::from(buf)
    }
}
