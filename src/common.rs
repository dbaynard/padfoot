//! Common data types

use std::{
    ffi::OsStr,
    fmt,
    fmt::Display,
    path::{Path, PathBuf},
};

use lopdf::Document;

use errors::*;
pub use these::*;

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
        write!(f, "\"{}\"", s)
    }
}

pub mod these {
    #[derive(Debug, PartialEq)]
    pub enum These<A, B> {
        This(A),
        That(B),
        These(A, B),
    }

    impl<A, B> These<A, B> {
        pub fn these<C>(
            self,
            this: impl FnOnce(A) -> C,
            that: impl FnOnce(B) -> C,
            these: impl FnOnce(A, B) -> C,
        ) -> C {
            use self::These::*;

            match self {
                This(a) => this(a),
                That(b) => that(b),
                These(a, b) => these(a, b),
            }
        }

        pub fn merge_these_with<C>(
            self,
            this: impl FnOnce(A) -> C,
            that: impl FnOnce(B) -> C,
            these: impl FnOnce(C, C) -> C,
        ) -> C {
            use self::These::*;

            match self {
                This(a) => this(a),
                That(b) => that(b),
                These(a, b) => these(this(a), that(b)),
            }
        }

        pub fn map<C>(self, f: impl FnOnce(B) -> C) -> These<A, C> {
            use self::These::*;

            match self {
                This(a) => This(a),
                That(b) => That(f(b)),
                These(a, b) => These(a, f(b)),
            }
        }

        pub fn bimap<C, D>(
            self,
            this: impl FnOnce(A) -> D,
            that: impl FnOnce(B) -> C,
        ) -> These<D, C> {
            use self::These::*;

            match self {
                This(a) => This(this(a)),
                That(b) => That(that(b)),
                These(a, b) => These(this(a), that(b)),
            }
        }

        pub fn do_this<E>(&self, this: impl FnOnce(&A) -> Result<(), E>) -> Result<(), E> {
            use self::These::*;

            match self {
                This(a) => this(a),
                That(_) => Ok(()),
                These(_, _) => Ok(()),
            }
        }

        pub fn do_that<E>(&self, that: impl FnOnce(&B) -> Result<(), E>) -> Result<(), E> {
            use self::These::*;

            match self {
                This(_) => Ok(()),
                That(b) => that(b),
                These(_, _) => Ok(()),
            }
        }
    }
}
