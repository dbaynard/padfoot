//! # Splitting and joining pdfs
//!

#[macro_use]
extern crate error_chain;

extern crate lopdf;

pub mod errors;

mod sel;
pub use sel::*;

/// The commands supplied to the library
#[derive(Debug)]
pub enum Command {
    Sel(InputSel),
}
