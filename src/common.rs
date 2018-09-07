//! Common data types

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct PDFName(PathBuf);

/// TODO
/// Ensure this corresponds to valid file
impl PDFName {
    pub fn new(pb: &Path) -> Self {
        PDFName(pb.to_path_buf())
    }

    pub fn over<A>(&self, f: impl FnOnce(&Path) -> A) -> A {
        f(&self.0)
    }
}

impl<'a> From<&'a Path> for PDFName {
    fn from(p: &Path) -> Self {
        PDFName::new(p)
    }
}

impl<'a> From<&'a OsStr> for PDFName {
    fn from(p: &OsStr) -> Self {
        PDFName(PathBuf::from(p))
    }
}
