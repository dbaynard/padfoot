//! An intermediate tree representation of PDFs

use std::{collections::btree_set::BTreeSet, str};

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
        let new_id = doc.new_object_id();
        let new_object = self.fold(doc);
        doc.objects.insert(new_id, new_object);
        new_id
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

        d.iter().for_each(|(s0, o)| {
            let s = s0.as_ref();

            dict.insert(
                s,
                match s {
                    "Parent" => PDFTree::Null,
                    _ => PDFTree::unfold(doc, seen, o),
                },
            );
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
