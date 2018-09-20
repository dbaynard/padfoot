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
        println!("File: {}", &name);

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

/// Burst pdf files into individual pages, named as the original, with a numerical suffix.
pub fn burst(input: &[PDFName]) -> Result<()> {
    let docs = input
        .iter()
        .filter_map(|name| name.load_doc().ok().map(|doc| (name, doc)));

    docs.map(|(name, doc)| -> Result<()> {
        println!();
        println!("File: {}", &name);

        let pages = doc.get_pages();

        let pp = page_range(&doc)?;

        let max_pages = match pp.into_inner() {
            (s, e) => {
                // Check underflow
                debug_assert!(e >= s);
                e + 1 - s
            }
        };

        let print_suffix_width = f64::ceil(f64::log10(max_pages as f64)) as usize;

        let name_prefix = name.file_stem();

        // Prefix and suffix, plus `_` and ".pdf"
        // TODO check overflow?
        let print_name_width = name_prefix.as_os_str().len() + print_suffix_width + 5;

        pages
            .iter()
            .map(|(no, &oid)| -> Result<()> {
                use std::fmt::Write;

                println!("Page {}", no);

                let mut new = Document::new();

                let pages_id = new.new_object_id();

                let old_page_d = doc
                    .get_dictionary(oid)
                    .error("Couldn’t locate page dictionary")?;

                let media_box = old_page_d
                    .get("MediaBox")
                    .error("Couldn’t get media box")?;

                let new_page = PDFTree::new(oid, &doc)?;

                let page_id = new_page.link_reference(&mut new);

                let pages = dictionary! {
                    "Type" => "Pages",
                    "Kids" => vec![page_id.into()],
                    "Count" => 1,
                    //"Resources" => resources_id,
                    "MediaBox" => media_box.clone(),
                };

                new.objects.insert(pages_id, Object::Dictionary(pages));

                new.get_object_mut(page_id)
                    .and_then(Object::as_dict_mut)
                    .map(|d| d.set("Parent", pages_id));

                let catalog_id = new.add_object(dictionary! {
                    "Type" => "Catalog",
                    "Pages" => pages_id,
                });

                new.trailer.set("Root", catalog_id);

                new.compress();

                // Could just use format! here but given we already know the size of the name, why not
                // do it explicitly.
                let mut new_name = String::with_capacity(print_name_width);
                write!(
                    new_name,
                    "{}_{:0width$}.pdf",
                    name_prefix.display(),
                    no,
                    width = print_suffix_width
                ).chain_err(|| "Couldn’t construct filename")?;

                new.save(new_name).chain_err(|| "Couldn’t save file")?;

                Ok(())
            })
            .for_each(drop);

        Ok(())
    }).for_each(drop);

    Ok(())
}
