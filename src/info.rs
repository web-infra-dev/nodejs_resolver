use crate::parse::Request;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Info {
    pub path: PathBuf,
    pub request: Request,
}

impl Info {
    pub fn from(path: PathBuf, request: Request) -> Self {
        Self { path, request }
    }

    pub fn get_path(&self) -> PathBuf {
        if self.request.target.is_empty() || self.request.target == "." {
            self.path.to_path_buf()
        } else {
            self.path.join(&*self.request.target)
        }
    }

    pub fn with_path(self, path: PathBuf) -> Self {
        Self { path, ..self }
    }

    pub fn with_target(self, target: &str) -> Self {
        let request = self.request.with_target(target);
        Self { request, ..self }
    }

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
