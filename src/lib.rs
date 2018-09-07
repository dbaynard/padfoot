//! # Splitting and joining pdfs
//!

#[macro_use]
extern crate error_chain;

extern crate itertools;

extern crate lopdf;

pub mod errors;
use errors::*;

mod sel;
pub use sel::*;

/// The commands supplied to the library
#[derive(Debug)]
pub enum Command {
    Sel(InputSel),
}

pub fn padfoot(c: Command) -> Result<()> {
    match c {
        Command::Sel(i) => sel(i),
    }
}
