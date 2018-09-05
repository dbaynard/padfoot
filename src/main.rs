#![recursion_limit = "1024"]

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
    let opt = Opt::from_args();

    let cmd = process_options(opt)?;

    println!("{:?}", cmd);

    Ok(())
}

fn process_options(opt: Opt) -> Result<Command> {

    match opt.cmd {

        OptCmd::Cat{mut inputs, output} => match output {

            Some(OutputCmd::Output{outfile}) => {
                let inputs = group_inputs(&inputs.inputs)?;
                let outfile = PDFName::new(&outfile);
                Ok(Command::Sel(Sel{inputs, outfile}))
            },

            None => {
                if let Some(InputElement::File(outf)) = inputs.inputs.pop() {
                    let inputs = group_inputs(&inputs.inputs)?;
                    let outfile = PDFName::new(&outf);
                    Ok(Command::Sel(Sel{inputs, outfile}))
                } else {
                    Err("Could not identify the output file.".into())
                }
            },

        }
    }
}

fn group_inputs(is: &[InputElement]) -> Result<Vec<PDFPages<PDFName>>> {

    is.iter().fold( Ok(vec!()), |mut rz, i| match i {

        InputElement::File(filepath) => {
            let _ = rz.as_mut().map(|z| z.push(
                PDFPages::new(
                    PDFName::new(filepath)
                )
            ));
            rz
        },

        InputElement::PageRange(range) => {
            let _ = rz.as_mut().map(|z| z.last_mut()
                .map(|l| l.push_range(range))
            );
            rz
        },

    })
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
