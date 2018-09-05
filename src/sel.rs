//! Select pages from pdf(s) and concatenate into a single output pdf

use std::{
    path::{
        Path,
        PathBuf,
    },
    ops::RangeInclusive,
};

/// The arguments supplied to the `sel` command.
pub type InputSel = Sel<PDFName>;

/// Data correspon to the `sel` command.
#[derive(Debug)]
pub struct Sel<A> {
    pub inputs: Vec<PDFPages<A>>,
    pub outfile: PDFName,
}

#[derive(Debug)]
pub struct PDFName (PathBuf);

impl PDFName {
    pub fn new(pb: &Path) -> Self {
        PDFName(pb.to_path_buf())
    }
}

#[derive(Debug)]
pub struct PDFPages<A> {
    /// Typically, this will be either a filename, document, or reference thereof.
    file: A,
    /// This relates to the `file`.
    /// An empty list corresponds to the full file.
    /// Otherwise, a list corresponds to the pages in the file.
    ///
    /// This method should not be exported.
    page_ranges: Vec<RangeInclusive<usize>>,
}

impl<A> PDFPages<A> {

    /// Create new `PDFPages` value corresponding to the full page range.
    pub fn new(file: A) -> PDFPages<A> {
        PDFPages {
            file,
            page_ranges: vec!(),
        }
    }

    pub fn push_range(&mut self, range: &RangeInclusive<usize>) {
        self.page_ranges.push(range.clone());
    }
}
