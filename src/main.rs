#![recursion_limit = "1024"]

/// Argument parsing uses `structopt`
#[macro_use]
extern crate structopt;
use structopt::StructOpt;
use std::ops::RangeInclusive;
use std::path::PathBuf;

extern crate combine;

extern crate padfoot;

use parsers::*;

fn main() {
    let opt = Opt::from_args();

    println!("{:?}", opt)
}

/// # Options
#[derive(Debug, StructOpt)]
enum Opt {
    #[structopt(name = "cat")]
    Cat {
        /// Input description
        #[structopt(parse(try_from_str = "parse_input_element"))]
        inputs: Vec<InputElement>,
        /// Output file, if present (otherwise stdout)
        #[structopt(raw(last = "true"), parse(from_os_str))]
        output: Option<PathBuf>,
    }
}

#[derive(Debug)]
pub enum InputElement {
    File(PathBuf),
    PageRange(RangeInclusive<usize>),
}

mod parsers {
    use combine::Parser;
    use combine::*;
    use combine::parser::char::*;
    use std::path::PathBuf;
    use padfoot::errors::Error;
    use InputElement;

    macro_rules! make_parser {
        ($name:ident, $output:ty, $body:block) => (
            fn $name<'a, I>() -> impl Parser<Input = I, Output = $output>
                where I: Stream<Item = char>,
                      <I as StreamOnce>::Error: ParseError<
                          <I as StreamOnce>::Item,
                          <I as StreamOnce>::Range,
                          <I as StreamOnce>::Position,
                      >,
                      //<I as StreamOnce>::Error: From<&'a str>,
            $body
        )
    }

    pub fn parse_input_element(i: &str) -> Result<InputElement, Error> {
        let (parsed, _) = input_element().parse(i)
            .or(Err("Couldn’t parse input element"))?;
        Ok(parsed)
    }

    make_parser!(input_element, InputElement,
    {
        choice!(
            page_range().map(|(f,t)| InputElement::PageRange(f ..= t)),
            path_buf().map(InputElement::File)
        )
    });

    make_parser!(path_buf, PathBuf,
    {
        not_followed_by(string("output"))
            .with(many1(any()))
            .map(|x: String| PathBuf::from(&x))
    });

    make_parser!(page_range, (usize, usize),
    {
        let page_range = number()
            .skip(char('-'))
            .and(number());
        page_range
    });

    make_parser!(number, usize,
    {
        many1(digit())
            .map(|x: String| x.parse().unwrap()
                //.or(Err("Couldn’t parse number from digits"))
            )
    });

}
