//! Select pages from pdf(s) and concatenate into a single output pdf

use std::ops::RangeInclusive;

use itertools::{Itertools, MinMaxResult};

use lopdf::*;

use common::*;
use errors::*;

/// The arguments supplied to the `sel` and `zip` commands.
pub type InputInOut = InOut<PDFName>;

/// Input files (with optional ranges) and output file corresponding to the `sel` and `zip`
/// commands.
#[derive(Debug)]
pub struct InOut<A> {
    pub inputs: Vec<PDFPages<A>>,
    pub outfile: PDFName,
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

    pub fn map<B>(self, f: impl FnOnce(A) -> B) -> PDFPages<B> {
        let page_ranges = self.page_ranges;
        let file = f(self.file);

        PDFPages { file, page_ranges }
    }

    pub fn traverse<B>(self, f: impl FnOnce(A) -> Result<B>) -> Result<PDFPages<B>> {
        let page_ranges = self.page_ranges;
        let file = f(self.file)?;

        Ok(PDFPages { file, page_ranges })
    }
}

impl PDFPages<PDFName> {
    fn load_doc(self) -> Option<PDFPages<Document>> {
        self.traverse(|x| PDFName::load_doc(&x)).ok()
    }
}

/// Run the input
pub fn sel(input: InputInOut) -> Result<()> {
    let sels = load_docs(input);

    //Ok(Document::new());

    Ok::<_, Error>(())
}

/// Display metadata
pub fn info(input: &[PDFName]) -> Result<()> {
    Err("Not implemented yet".into())
}

#[derive(Debug)]
enum DocsForLoad<'a> {
    InOnly(&'a [PDFName]),
    InOut(&'a InputInOut),
}

/*
 *impl DocsForLoad<'a> {
 *    fn load_docs(self) ->  {
 *        let inputs = match
 *    }
 *}
 */

/// Load specified documents
///
/// TODO Don’t silence errors
fn load_docs(inps: InputInOut) -> InOut<Document> {
    type POut = PDFPages<Document>;

    let inputs = inps.inputs;
    let outfile = inps.outfile;

    let inputs: Vec<POut> = inputs.into_iter().filter_map(PDFPages::load_doc).collect();

    InOut { inputs, outfile }
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
