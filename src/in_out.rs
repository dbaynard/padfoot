//! Select pages from pdf(s) and concatenate into a single output pdf

use std::{ops::RangeInclusive, result, str};

use itertools::{Itertools, MinMaxResult};
use xmlparser::{Token, Tokenizer};

use lopdf::{Stream, *};

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
        self.traverse(|x| x.load_doc()).ok()
    }
}

/// Run the input
pub fn sel(input: InputInOut) -> Result<()> {
    //let sels = load_docs(input);

    //Ok(Document::new());

    Ok::<_, Error>(())
}

/// Display metadata
pub fn info(input: &[PDFName]) -> Result<()> {
    let docs = input.iter().filter_map(|x| x.load_doc().ok());

    docs.for_each(|x| println!("{:#?}", get_metadata(&x)));

    Ok(())
}

fn get_metadata(doc: &Document) -> Result<(&Dictionary, Vec<Token>)> {
    let trail = &doc.trailer;
    let info = trail
        .get("Info")
        .and_then(Object::as_reference)
        .error("Couldn’t identify pdf info")
        .and_then(|r| doc.get_dictionary(r).error("Couldn’t access pdf info"))?;

    let catalog = doc.catalog().error("Couldn’t access catalog")?;

    let metadata = catalog
        .get("Metadata")
        .and_then(Object::as_reference)
        .error("Couldn’t identify metadata")
        .and_then(|r| {
            doc.get_object(r)
                .and_then(Object::as_stream)
                .error("Couldn’t access metadata")
        })
        .and_then(|s| str::from_utf8(&s.content).error("Couldn’t decode utf8"))
        .map(Tokenizer::from)
        .and_then(|x| {
            x.collect::<result::Result<Vec<_>, _>>()
                .error("Couldn’t parse xml")
        })?;

    fn decode_stream(s: &Stream) -> Result<content::Content> {
        s.decode_content().error("Couldn’t parse content stream")
    }

    Ok((info, metadata))
}

/*
 *#[derive(Debug)]
 *struct DocsForLoad<'a> (&'a [PDFName]);
 *
 *impl<'a> DocsForLoad<'a> {
 *    pub fn new(ds: &'a [PDFName]) -> DocsForLoad<'a> {
 *        DocsForLoad (ds)
 *    }
 *
 *    fn load_all(self) -> impl Iterator<Item = Result<Document>> {
 *        self.0.iter().map(|x| PDFName::load_doc(x))
 *    }
 *}
 */

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
