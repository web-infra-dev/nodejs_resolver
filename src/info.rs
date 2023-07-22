#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
use std::{borrow::Cow, path::Path, sync::Arc};

use path_absolutize::Absolutize;

use crate::parse::Request;

#[cfg(windows)]
fn has_trailing_slash(p: &Path) -> bool {
    let last = p.as_os_str().encode_wide().last();
    last == Some(b'\\' as u16) || last == Some(b'/' as u16)
}
#[cfg(unix)]
fn has_trailing_slash(p: &Path) -> bool {
    p.as_os_str().as_bytes().last() == Some(&b'/')
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedPath(Arc<Path>);

impl NormalizedPath {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        // perf: this method does not re-allocate memory if the path does not contain any dots.
        let normalized = path.as_ref().absolutize_from(Path::new("")).unwrap();
        let path = if has_trailing_slash(path.as_ref()) {
            Path::new(&format!("{}/", normalized.display())).into()
        } else {
            normalized.into()
        };
        NormalizedPath(path)
    }
}

impl AsRef<Path> for NormalizedPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct Info {
    path: NormalizedPath,
    request: Request,
}

impl From<NormalizedPath> for Info {
    fn from(value: NormalizedPath) -> Self {
        Info { path: value, request: Default::default() }
    }
}

impl Info {
    #[must_use]
    pub fn new<P: AsRef<Path>>(path: P, request: Request) -> Self {
        Self { path: NormalizedPath::new(path), request }
    }

    #[must_use]
    pub fn with_path<P: AsRef<Path>>(self, path: P) -> Self {
        Self { path: NormalizedPath::new(path), ..self }
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
    pub fn normalized_path(&self) -> &NormalizedPath {
        &self.path
    }

    #[must_use]
    pub fn request(&self) -> &Request {
        &self.request
    }

    #[must_use]
    pub fn to_resolved_path(&self) -> Cow<'_, Path> {
        if self.request.target().is_empty() || self.request.target() == "." {
            Cow::Borrowed(&self.path.0)
        } else {
            let p = NormalizedPath::new(self.path.as_ref().join(self.request.target()));
            Cow::Owned(p.0.to_path_buf())
        }
    }
}
