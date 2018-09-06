//! Select pages from pdf(s) and concatenate into a single output pdf

use std::{
    ops::RangeInclusive,
    path::{Path, PathBuf},
};

use itertools::{Itertools, MinMaxResult};

use lopdf::*;

use errors::*;

/// The arguments supplied to the `sel` command.
pub type InputSel = Sel<PDFName>;

/// Data corresponding to the `sel` command.
#[derive(Debug)]
pub struct Sel<A> {
    pub inputs: Vec<PDFPages<A>>,
    pub outfile: PDFName,
}

#[derive(Debug)]
pub struct PDFName(PathBuf);

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
            page_ranges: vec![],
        }
    }

    pub fn push_range(&mut self, range: &RangeInclusive<usize>) {
        self.page_ranges.push(range.clone());
    }
}

/// Load specified documents
pub fn load_docs(inps: InputSel) -> Sel<Document> {
    type PIn = PDFPages<PDFName>;
    type POut = PDFPages<Document>;

    fn load_doc(PDFPages { file, page_ranges }: PIn) -> Option<POut> {
        Document::load(&file.0)
            .map(|file| PDFPages { file, page_ranges })
            .ok()
    }

    let inputs = inps.inputs;
    let outfile = inps.outfile;

    let inputs: Vec<POut> = inputs.into_iter().filter_map(load_doc).collect();

    Sel { inputs, outfile }
}

/// Identify a document’s page range
pub fn page_range(doc: &Document) -> Result<RangeInclusive<u32>> {
    let pages = doc.get_pages();

    match pages.keys().minmax() {
        // TODO Should assert no error here
        MinMaxResult::NoElements => Err("No pages in pdf".into()),
        MinMaxResult::OneElement(&el) => Ok(el..=el),
        MinMaxResult::MinMax(&min, &max) => Ok(min..=max),
    }
}
