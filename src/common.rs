//! Common data types

use std::{
    ffi::OsStr,
    fmt,
    fmt::Display,
    path::{Path, PathBuf},
};

use lopdf::Document;

use errors::*;

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

    pub fn load_doc(&self) -> Result<Document> {
        self.over(|x| Document::load(x))
            .or_else(|_| Err("Couldn’t load document".into()))
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

impl Display for PDFName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = self.0.to_str().ok_or_else(|| fmt::Error::default())?;
        write!(f, "{}", s)
    }
}
