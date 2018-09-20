//! Process pdfs

use std::{collections::btree_set::BTreeSet, ops::RangeInclusive, str, string::String};

use chrono::{DateTime, NaiveDateTime};
use itertools::{Itertools, MinMaxResult};
use linked_hash_map::LinkedHashMap;

use lopdf::*;

use errors::*;

#[derive(Clone, Debug)]
/// A Tree representation of the pdf tree, for extracting subtrees of the `lopdf` representation
/// (e.g. pages, with all associated data).
///
/// Create from the `lopdf` structure using `PDFTree::new` and fold into a `lopdf::Document` using
/// `PDFTree::link_reference`.
///
/// There are still references within the tree, produced when there are duplicate references in the
/// source representation, so each object is represented exactly once in the tree directly, and
/// possibly more often as a reference.
///
/// TODO This is _extremely_ hacky, and the solution may be reference counting for sub trees.
///
/// This type recurses mutually with `PDFDictionary`.
///
/// The lifetime associated with this data structure corresponds to the existing document which
/// acts as the source.
pub enum PDFTree<'a> {
    Null,
    Boolean(bool),
    Integer(i64),
    Real(f64),
    Name(&'a [u8]),
    String(&'a [u8], &'a StringFormat),
    Array(Vec<Box<PDFTree<'a>>>),
    Dictionary(Box<PDFDictionary<'a>>),
    Stream(&'a Stream),
    Reference(ObjectId),
    SubTree(Box<PDFTree<'a>>),
}

impl<'a> PDFTree<'a> {
    /// Create a `PDFTree` from an `lopdf::Object`.
    ///
    /// As the `lopdf::Object` may include a reference, this function instead takes a reference to a
    /// `lopdf::Document` and an `lopdf::ObjectId`.
    ///
    /// This returns `Err` if the `lopdf::Document` does not contain that `lopdf::ObjectId`.
    pub fn new(oid: ObjectId, doc: &'a Document) -> Result<Self> {
        let o = doc.get_object(oid).error("Couldn’t locate page object")?;

        let mut seen = BTreeSet::new();
        seen.insert(oid);

        Ok(PDFTree::unfold(doc, &mut seen, o))
    }

    fn unfold(doc: &'a Document, seen: &mut BTreeSet<ObjectId>, o: &'a Object) -> Self {
        match o {
            Object::Null => PDFTree::Null,
            Object::Boolean(b) => PDFTree::Boolean(*b),
            Object::Integer(i) => PDFTree::Integer(*i),
            Object::Real(f) => PDFTree::Real(*f),
            Object::Name(v) => PDFTree::Name(v.as_ref()),
            Object::String(v, f) => PDFTree::String(v.as_ref(), f),
            Object::Array(v) => {
                let arr = v
                    .iter()
                    .map(|x| Box::new(PDFTree::unfold(doc, seen, x)))
                    .collect();
                PDFTree::Array(arr)
            }
            Object::Dictionary(d) => {
                PDFTree::Dictionary(Box::new(PDFDictionary::new(doc, seen, d)))
            }
            Object::Stream(s) => PDFTree::Stream(&s),
            Object::Reference(oid) => match seen.contains(oid) {
                // TODO Note that this is a reference to the object in the *Old* structure
                true => PDFTree::Reference(*oid),
                false => {
                    seen.insert(*oid);
                    doc.get_object(*oid)
                        .map(|x| PDFTree::SubTree(Box::new(PDFTree::unfold(doc, seen, x))))
                        .unwrap_or_else(|| PDFTree::Null)
                }
            },
        }
    }

    /// Fold a `PDFTree` into the supplied `lopdf::Document`, providing the `lopdf::ObjectId` of
    /// the `lopdf::Object` corresponding to the root of the `PDFTree`.
    pub fn link_reference(&self, doc: &mut Document) -> ObjectId {
        let new_object = self.fold(doc);
        doc.add_object(new_object)
    }

    fn fold(&self, doc: &mut Document) -> Object {
        match self {
            PDFTree::Null => Object::Null,
            PDFTree::Boolean(b) => Object::Boolean(*b),
            PDFTree::Integer(i) => Object::Integer(*i),
            PDFTree::Real(f) => Object::Real(*f),
            PDFTree::Name(v) => Object::Name(v.to_vec()),
            PDFTree::String(v, f) => Object::String(v.to_vec(), (*f).clone()),
            PDFTree::Array(v) => {
                let arr = v.iter().map(|x| x.fold(doc)).collect();
                Object::Array(arr)
            }
            PDFTree::Dictionary(d) => d.fold(doc),
            PDFTree::Stream(s) => Object::Stream((*s).clone()),
            // TODO This is wrong; it inserts a reference to an object in the old structure.
            PDFTree::Reference(oid) => Object::Reference(*oid),
            PDFTree::SubTree(tree) => {
                let oid = tree.link_reference(doc);
                Object::Reference(oid)
            }
        }
    }
}

#[derive(Clone, Debug)]
/// A `LinkedHashMap` of references to the key–value pairs in a `lopdf::Dictionary`.
///
/// This is much like a `lopdf::Dictionary` except it only contains references, and it forms part
/// of the mutually recursive structure with the `PDFTree`.
///
/// TODO It currently `/Parent` keys. These should be propagated (somehow).
///
/// Like for the `PDFTree`, the lifetime corresponds to the lifetime of the associated
/// `lopdf::Document`.
pub struct PDFDictionary<'a>(LinkedHashMap<&'a str, PDFTree<'a>>);

impl<'a> PDFDictionary<'a> {
    fn new(doc: &'a Document, seen: &mut BTreeSet<ObjectId>, d: &'a Dictionary) -> Self {
        let mut dict = LinkedHashMap::new();

        d.iter().for_each(|(s, o)| {
            if s != "Parent" {
                dict.insert(s.as_ref(), PDFTree::unfold(doc, seen, o));
            }
        });

        PDFDictionary(dict)
    }

    fn fold(&self, doc: &mut Document) -> Object {
        let mut dict = Dictionary::new();
        self.0
            .iter()
            .for_each(|(&s, tree)| dict.set(s.clone(), tree.fold(doc)));
        Object::Dictionary(dict)
    }
}

/// Pretty print a simple object
pub fn simple_display_object<'a>(doc: &'a Document, o: &'a Object) -> Result<String> {
    use lopdf::Object::*;
    use std::string::String;

    match o {
        Null => Ok(String::from("")),
        Boolean(ref b) => Ok(format!("{}", b)),
        Integer(ref i) => Ok(format!("{}", i)),
        Real(ref f) => Ok(format!("{}", f)),
        Name(v) => String::from_utf8(v.clone()).chain_err(|| "Could not convert as utf8 name"),
        String(v, _fmt) => {
            let s = String::from_utf8_lossy(v);
            display_trail_date(&s).or_else(|_| Ok(s.to_string()))
        }
        Array(v) => Ok(v
            .into_iter()
            .map(|x| simple_display_object(doc, x))
            .filter_map(|x| x.ok())
            .join(",\n")),
        Dictionary(_) => Err("Dictionary".into()),
        Stream(_) => Err("Stream".into()),
        Reference(r) => {
            let v = doc.get_object(*r).error("Couldn’t follow reference")?;
            // TODO take care of recursion, somehow?
            simple_display_object(doc, v)
        }
    }
}

/// An iterator of the trail’s contents
pub fn get_trail_info(doc: &Document) -> Result<impl Iterator<Item = (&str, &Object)>> {
    let trail = &doc.trailer;

    let info = trail
        .get("Info")
        .and_then(Object::as_reference)
        .error("Couldn’t identify pdf info")
        .and_then(|r| doc.get_dictionary(r).error("Couldn’t access pdf info"))?;

    Ok(info.iter().map(|(s, o)| (&s[..], o)))
}

/// Identify a document’s page range
pub fn page_range(doc: &Document) -> Result<RangeInclusive<u32>> {
    let pages = doc.get_pages();

    match pages.keys().minmax() {
        // TODO Should assert no error here
        MinMaxResult::NoElements => Err("No pages in pdf".into()),
        MinMaxResult::OneElement(&el) => Ok(el..=el),
        // TODO need to ensure max ≥ min
        MinMaxResult::MinMax(&min, &max) => Ok(min..=max),
    }
}

/*
 *pub fn get_metadata(doc: &Document) -> Result<Vec<(String, String)>> {
 *    let catalog = doc.catalog().error("Couldn’t access catalog")?;
 *
 *    let metadata = catalog
 *        .get("Metadata")
 *        .and_then(Object::as_reference)
 *        .error("Couldn’t identify metadata")
 *        .and_then(|r| {
 *            doc.get_object(r)
 *                .and_then(Object::as_stream)
 *                .error("Couldn’t access metadata")
 *        })
 *        .map(|s| &s.content)
 *        //.and_then(|s| str::from_utf8(&s[54..]).error("Couldn’t decode utf8"))?;
 *        .and_then(|s| Element::parse(&s[54..]).chain_err(|| "Couldn’t read xml"))
 *        .map(|e| text_names(&e).into_iter().map(|(n,t)| (n.into_owned(), t.into_owned())).collect())?;
 *
 *        //fn decode_stream(s: &Stream) -> Result<content::Content> {
 *            //s.decode_content().error("Couldn’t parse content stream")
 *        //}
 *
 *        //fn chain_leaves<A>(e: &Element) -> impl Iterator<Item = &Element> {
 *            //match e.children[..] {
 *                //[] => iter::once(e),
 *                ////ref cs => cs.iter().fold(iter::empty(), |a, c| a.chain(chain_leaves(c)).collect()),
 *                //// TODO
 *                //ref cs => cs.iter().flat_map(|it| it.clone()a.chain(chain_leaves(c)).collect()),
 *            //}
 *        //}
 *
 *    fn fold_element_leaves<'a, A>(e: &'a Element, f: impl Fn(&'a Element) -> A) -> Vec<A> {
 *        match e.children[..] {
 *            [] => vec![f(e)],
 *            ref cs => cs
 *                .iter()
 *                .fold(vec![], |_a: Vec<A>, c: &Element| fold_element_leaves(c, &f)),
 *        }
 *    }
 *
 *    fn text_names<'a>(el: &'a Element) -> Vec<(Cow<'a, str>, Cow<'a, str>)> {
 *        fold_element_leaves(el, text_name)
 *            .into_iter()
 *            .filter_map(|x| x)
 *            .collect()
 *    }
 *
 *    fn text_name<'a>(e: &'a Element) -> Option<(Cow<'a, str>, Cow<'a, str>)> {
 *        e.text
 *            .as_ref()
 *            .map(|ref t| (Cow::from(&e.name), Cow::from(&t[..])))
 *    }
 *
 *    Ok(metadata)
 *}
 */

/// Pretty print a date, formatted in the pdf trailer
fn display_trail_date(s: &str) -> Result<String> {
    DateTime::parse_from_str(&(s.replace("'", "").replace("Z", "+")), "D:%Y%m%d%H%M%S%z")
        .map(|d| format!("{}", d.format("%a, %d %b %Y %T %z")))
        .or_else(|_| {
            NaiveDateTime::parse_from_str(s, "D:%Y%m%d%H%M%S")
                .map(|d| format!("{}", d.format("%a, %d %b %Y %T")))
        })
        .chain_err(|| "Couldn’t parse date")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_trail_date() {
        assert_eq!(
            display_trail_date("D:20170712171035+01'00'").unwrap_or_else(|e| format!("{:?}", e)),
            "Wed, 12 Jul 2017 17:10:35 +0100"
        );
        assert_eq!(
            display_trail_date("D:20170711121931").unwrap_or_else(|e| format!("{:?}", e)),
            "Tue, 11 Jul 2017 12:19:31"
        );
        assert_eq!(
            display_trail_date("D:20180710153507Z00'00'").unwrap_or_else(|e| format!("{:?}", e)),
            "Tue, 10 Jul 2018 15:35:07 +0000"
        );
    }
}
