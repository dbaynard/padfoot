#![recursion_limit = "1024"]

/// Argument parsing uses `structopt`
#[macro_use]
extern crate structopt;
use structopt::StructOpt;
use std::ops::Range;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

extern crate combine;

extern crate padfoot;
use padfoot::*;
use padfoot::errors::*;

fn main() {
    let opt = Opt::from_args();

    println!("{:?}", opt)
}

/// # Options
#[derive(Debug, StructOpt)]
struct Opt {
    /// Operation
    operation: Operation,
    /// Input description
    inputs: Vec<InputElement>,
    /// Output file, if present (otherwise stdout)
    #[structopt(raw(last = "true"), parse(from_os_str))]
    output: Option<PathBuf>,
}

#[derive(Debug)]
enum Operation {
    Cat,
}

impl FromStr for Operation {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "cat" => Ok(Operation::Cat),
            _ => Err("Couldn’t identify operation".into()),
        }
    }
}

#[derive(Debug)]
enum InputElement {
    File(&'static Path),
    PageRange(Range<usize>),
}

impl FromStr for InputElement {
    type Err = Error;

    fn from_str(_s: &str) -> std::result::Result<Self, Self::Err> {
        Err("Couldn’t discern input".into())
    }
}

mod parsers {
    use combine::Parser;
    use combine::*;
    use combine::parser::char::*;

    macro_rules! make_parser {
        ($name:ident, $output:ty, $body:block) => (
            fn $name<I>() -> impl Parser<Input = I, Output = $output>
                where I: Stream<Item = char>,
                      <I as StreamOnce>::Error: ParseError<
                          <I as StreamOnce>::Item,
                          <I as StreamOnce>::Range,
                          <I as StreamOnce>::Position,
                      >,
                      <I as StreamOnce>::Error: From<&'static str>,
            $body
        )
    }

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
            .map(as_string)
            .flat_map(|x| str::parse(&x)
                .or(Err("Couldn’t parse number from digits".into()))
                )
    });

    fn as_string(v: Vec<char>) -> String {
        v.into_iter().collect()
    }
}
