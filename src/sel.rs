//! Select pages from pdf(s) and concatenate into a single output pdf

use std::{
    path::PathBuf,
    ops::RangeInclusive,
};

/// The arguments supplied to the `sel` command.
pub type InputSel = Sel<PDFName>;

/// Data correspon to the `sel` command.
pub struct Sel<A> {
    pub inputs: PDFPages<A>,
    pub output: PDFName,
}

pub struct PDFName(PathBuf);

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
