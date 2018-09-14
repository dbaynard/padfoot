//! Select pages from pdf(s) and concatenate into a single output pdf

use std::{fmt, fmt::Display, marker, ops::RangeInclusive};

use lopdf::*;

use common::*;
use errors::*;
use pdf::*;

/// The arguments supplied to the `sel` and `zip` commands.
pub type InputsWithOutputSpec = InputsWithOutput<NotLoaded>;

impl Display for InputsWithOutputSpec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inputs
            .iter()
            .map(|i| write!(f, " {}", i))
            .collect::<fmt::Result>()?;
        write!(f, " output {}", self.outfile)
    }
}

/// Input files (with optional ranges) and output file corresponding to the `sel` and `zip`
/// commands.
#[derive(Debug)]
pub struct InputsWithOutput<T> {
    pub inputs: Vec<PDFPages<T>>,
    pub outfile: PDFName,
}

type File = These<PDFName, Document>;

#[derive(Debug)]
pub struct PDFPages<T> {
    /// This will be either a filename, document, or both.
    file: File,
    /// This relates to the `file`.
    /// An empty list corresponds to the full file.
    /// Otherwise, a list corresponds to the pages in the file.
    ///
    /// This method should not be exported.
    page_ranges: Vec<RangeInclusive<usize>>,
    _marker: marker::PhantomData<T>,
}

#[derive(Debug)]
pub enum NotLoaded {}

#[derive(Debug)]
pub enum Loaded {}

/// The specification
pub type PDFPagesSpec = PDFPages<NotLoaded>;
pub type PDFPagesLoad = PDFPages<Loaded>;

impl Display for PDFPagesSpec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.file.do_this(|name| write!(f, " {}", name))?;
        self.page_ranges
            .iter()
            .cloned()
            .map(RangeInclusive::into_inner)
            .map(|(fr, to)| (format!("{}", fr), format!("{}", to)))
            .map(|(fr, to)| write!(f, " {}-{}", fr, to))
            .collect()
    }
}

impl PDFPagesSpec {
    /// Create new `PDFPages` value corresponding to the full page range.
    pub fn new(name: PDFName) -> PDFPagesSpec {
        PDFPages {
            file: These::This(name),
            page_ranges: vec![],
            _marker: marker::PhantomData,
        }
    }

    pub fn load_doc(self) -> Option<PDFPagesLoad> {
        // TODO This should never be anything but `This`. How to statically guarantee?
        let file = self.file.these(
            |x| x.load_doc().ok().map(|y| These::These(x, y)),
            |_| None,
            |_, _| None,
        )?;

        Some(PDFPages {
            file,
            page_ranges: self.page_ranges,
            _marker: marker::PhantomData,
        })
    }
}

impl<T> PDFPages<T> {
    pub fn push_range(&mut self, range: &RangeInclusive<usize>) {
        self.page_ranges.push(range.clone());
    }

    pub fn map(self, f: impl FnOnce(File) -> File) -> PDFPages<T> {
        let file = f(self.file);

        PDFPages {
            file,
            page_ranges: self.page_ranges,
            _marker: marker::PhantomData,
        }
    }

    pub fn traverse(self, f: impl FnOnce(File) -> Result<File>) -> Result<PDFPages<T>> {
        let file = f(self.file)?;

        Ok(PDFPages {
            file,
            page_ranges: self.page_ranges,
            _marker: marker::PhantomData,
        })
    }
}

/// Run the input
pub fn sel(_input: InputsWithOutputSpec) -> Result<()> {
    //let sels = load_docs(input);

    //Ok(Document::new());

    Ok::<_, Error>(())
}

/// Display metadata
pub fn info(input: &[PDFName]) -> Result<()> {
    let docs = input
        .iter()
        .filter_map(|name| name.load_doc().ok().map(|doc| (name, doc)));

    docs.map(|(name, doc)| -> Result<()> {
        println!();
        println!("File: {}", name);

        let i = get_trail_info(&doc)?;

        i.filter_map(|(k, v)| {
            let d = simple_display_object(&doc, v).ok()?;
            Some((k, d))
        }).for_each(|(k, v)| println!("{}: {}", k, v));

        let p = page_range(&doc).map(RangeInclusive::into_inner)?;

        println!("Pages: {}–{}", p.0, p.1);

        Ok(())
    }).for_each(drop);

    Ok(())
}
