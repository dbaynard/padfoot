//! Process pdfs

use std::{ops::RangeInclusive, str, string::String};

use chrono::{DateTime, NaiveDateTime};
use itertools::{Itertools, MinMaxResult};

use lopdf::*;

use errors::*;

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
