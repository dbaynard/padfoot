#![recursion_limit = "1024"]

use std::path::Path;

/// Argument parsing uses `structopt`
#[macro_use]
extern crate structopt;
use structopt::StructOpt;

extern crate combine;

extern crate padfoot;
use padfoot::{
    *,
    errors::*,
};

use options::*;

fn main() -> Result<()> {
    // Why mutable? Well, it means they can be normalized later.
    let mut opt = Opt::from_args();

    let cmd = process_options(&mut opt)?;

    println!("{:?}", cmd);

    Ok(())
}

/// The options supplied to the program must be converted to the internal DSL it uses.
///
/// The list of valid options settings according to the structopt library does not match the valid
/// commands.
fn process_options(opt: &mut Opt) -> Result<Command> {

    match opt.cmd {
        OptCmd::Cat{ref mut inputs, ref output} =>
            normalize_inputs(inputs, output, Command::Sel)
    }
}

/// It is possible to supply a list of inputs, with the output last (rather than delimited with the
/// `output` symbol, like pdftk). This ensures that there is exactly one output file.
fn normalize_inputs(
    inp: &mut Inputs,
    output: &Option<OutputCmd>,
    f: impl Fn(InputSel) -> Command,
) -> Result<Command>
{
    let inputs = &mut inp.inputs;

    let outfile = output.as_ref()
        .map(|OutputCmd::Output{outfile}| outfile.clone())
        .ok_or_else::<Error,_>(|| "No explicit output supplied".into())
        // If no explicit output, pop the last input value
        .or_else(|_| inputs.pop()
            .ok_or_else::<Error,_>(|| "No input supplied.".into())
            .and_then(|x| match x {
                InputElement::File(outfile) => Ok(outfile),
                _ => Err("The arguments must finish with an output file name.".into()),
            })
        )?;

    let sel = apply_inputs(
        &inputs,
        &outfile,
        |inputs, outfile| Sel{inputs, outfile}
    )?;

    Ok(f(sel))
}

fn apply_inputs<A>(
    inputs: &[InputElement],
    outfile: &Path,
    f: impl Fn(Vec<PDFPages<PDFName>>, PDFName) -> A
) -> Result<A>
{
    let inputs = group_inputs(inputs)?;
    let outfile = PDFName::new(outfile);
    Ok(f(inputs, outfile))
}

/// The input list contains a mix of filenames and page ranges.
///
/// The list must begin with a filename.
/// Each filename may be followed by a (possibly empty) list of page ranges.
/// These ranges are associated with the most recent preceding filename.
fn group_inputs(is: &[InputElement]) -> Result<Vec<PDFPages<PDFName>>> {

    let input_algebra = |mut rz: Result<Vec<_>>, i: &InputElement| match i {

        InputElement::File(filepath) => {
            let _ = rz.as_mut().map(|z| z.push(
                PDFPages::new(
                    PDFName::new(&filepath)
                )
            ));
            rz
        },

        InputElement::PageRange(range) => {
            let _ = rz.as_mut().map(|z| z.last_mut()
                .map(|l| l.push_range(&range))
            );
            rz
        },

    };

    is.iter().fold( Ok(vec!()), input_algebra)
}

/// StructOpt option types corresponding to the CLI interface
mod options {
    use std::{
        ops::RangeInclusive,
        path::PathBuf,
    };

    use parsers::*;

    /// # Options
    #[derive(Debug, StructOpt)]
    pub struct Opt {
        /// Command
        #[structopt(subcommand)]
        pub cmd: OptCmd,
    }

    #[derive(Debug, StructOpt)]
    pub enum OptCmd {
        #[structopt(name = "cat")]
        Cat {
            #[structopt(flatten)]
            inputs: Inputs,
            #[structopt(subcommand)]
            output: Option<OutputCmd>,
        }
    }

    #[derive(Debug, StructOpt)]
    pub struct Inputs {
        /// Input description
        #[structopt(parse(try_from_str = "parse_input_element"))]
        pub inputs: Vec<InputElement>,
    }

    #[derive(Debug, StructOpt)]
    pub enum OutputCmd {
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
}

/// Option parsing
mod parsers {
    use combine::{
        *,
        char::*,
    };

    use std::path::PathBuf;

    use padfoot::errors::Error;

    use options::*;

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
