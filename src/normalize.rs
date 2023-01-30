use path_absolutize::Absolutize;

use std::{borrow::Cow, path::Path};

pub trait NormalizePath {
    fn normalize(&self) -> Cow<Path>;
}

impl NormalizePath for Path {
    #[inline]
    fn normalize(&self) -> Cow<Path> {
        // perf: this method does not re-allocate memory if the path does not contain any dots.
        self.absolutize_from(Path::new("")).unwrap()
    }
}
