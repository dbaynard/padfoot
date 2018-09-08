//! # Splitting and joining pdfs
//!

#[macro_use]
extern crate error_chain;

extern crate itertools;

extern crate chrono;
extern crate xmltree;

extern crate lopdf;

pub mod errors;
use errors::*;

mod common;
pub use common::*;

mod in_out;
pub use in_out::*;

/// The commands supplied to the library
#[derive(Debug)]
pub enum Command {
    Sel(InputInOut),
    Zip(InputInOut),
    Burst(Vec<PDFName>),
    Info(Vec<PDFName>),
}

pub fn padfoot(c: Command) -> Result<()> {
    match c {
        Command::Sel(i) => sel(i),
        Command::Zip(_) => Err("Not implemented yet".into()),
        Command::Burst(_) => Err("Not implemented yet".into()),
        Command::Info(i) => info(&i),
    }
}
