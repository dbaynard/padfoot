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
struct Opt {
    /// Command
    #[structopt(subcommand)]
    cmd: OptCmd,
}

#[derive(Debug, StructOpt)]
enum OptCmd {
    #[structopt(name = "cat")]
    Cat {
        #[structopt(flatten)]
        inputs: Inputs,
        #[structopt(subcommand)]
        output: Option<OutputCmd>,
    }
}

#[derive(Debug, StructOpt)]
struct Inputs {
    /// Input description
    #[structopt(parse(try_from_str = "parse_input_element"))]
    inputs: Vec<InputElement>,
}

#[derive(Debug, StructOpt)]
enum OutputCmd {
    #[structopt(name = "output")]
    Output {
        #[structopt(parse(from_os_str))]
        outfile: PathBuf,
    },
}

#[derive(Debug)]
pub enum InputElement {
    File(PathBuf),
    PageRange(RangeInclusive<usize>),
}

mod parsers {
    use combine::{
        *,
        char::*,
    };

    use std::path::PathBuf;

    use padfoot::errors::Error;

    use InputElement;

    /// Create a parser. This simply handles the messy, messy types.
    macro_rules! make_parser {
        ($name:ident, $output:ty, $body:block) => (
            fn $name<'a, I>() -> impl Parser<Input = I, Output = $output>
                where I: Stream<Item = char>,
                      <I as StreamOnce>::Error: ParseError<
                          <I as StreamOnce>::Item,
                          <I as StreamOnce>::Range,
                          <I as StreamOnce>::Position,
                      >,
            $body
        )
    }

    /// Parse a single input element
    pub fn parse_input_element(i: &str) -> Result<InputElement, Error> {
        let (parsed, _) = input_element().parse(i)
            .or(Err("Couldn’t parse input element"))?;
        Ok(parsed)
    }

    make_parser!(input_element, InputElement,
    {
        choice!(
            try(inclusive_range()).map(|(f,t)| InputElement::PageRange(f ..= t)),
            path_buf().map(InputElement::File)
        ).message("Couldn’t parse input element")
    });

    make_parser!(path_buf, PathBuf,
    {
        many1(any())
            .map(|x: String| PathBuf::from(&x))
            .message("Couldn’t parse potential path")
    });

    make_parser!(inclusive_range, (usize, usize),
    {
        let inclusive_range = number()
            .skip(char('-'))
            .and(number());
        inclusive_range
            .message("Couldn’t parse inclusive range")
    });

    make_parser!(number, usize,
    {
        from_str(many1::<String, _>(digit()))
            .message("Couldn’t parse number from digits")
    });

}
