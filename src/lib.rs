//! # Splitting and joining pdfs
//!

#[macro_use]
extern crate error_chain;

extern crate itertools;

extern crate lopdf;

pub mod errors;
use errors::*;

mod common;
pub use common::*;

mod sel;
pub use sel::*;

/// The commands supplied to the library
#[derive(Debug)]
pub enum Command {
    Sel(InputSel),
    Zip(InputSel),
    Burst(Vec<PDFName>),
    Info(Vec<PDFName>),
}

pub fn padfoot(c: Command) -> Result<()> {
    match c {
        Command::Sel(i) => sel(i),
        Command::Zip(_) => Err("Not implemented yet".into()),
        Command::Burst(_) => Err("Not implemented yet".into()),
        Command::Info(_) => Err("Not implemented yet".into()),
    }
}
