//! # Splitting and joining pdfs
//!

#[macro_use]
extern crate error_chain;

extern crate itertools;

extern crate chrono;
extern crate xmltree;

extern crate lopdf;

use std::{fmt, fmt::Display};

pub mod errors;
use errors::*;

mod common;
pub use common::*;

mod in_out;
pub use in_out::*;

mod pdf;

/// The commands supplied to the library
#[derive(Debug)]
pub enum Command {
    Sel(InputsWithOutputSpec),
    Zip(InputsWithOutputSpec),
    Burst(Vec<PDFName>),
    Info(Vec<PDFName>),
}

impl Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Command::*;

        write!(f, "{}", "padfoot ")?;

        match self {
            Sel(i) => write!(f, "sel{}", i),
            Zip(i) => write!(f, "zip{}", i),
            Burst(i) => {
                write!(f, "burst")?;
                i.into_iter().map(|x| write!(f, " {}", x)).collect()
            }
            Info(i) => {
                write!(f, "info")?;
                i.into_iter().map(|x| write!(f, " {}", x)).collect()
            }
        }
    }
}

pub fn padfoot(c: Command) -> Result<()> {
    match c {
        Command::Sel(i) => sel(i),
        Command::Zip(_) => Err("Not implemented yet".into()),
        Command::Burst(_) => Err("Not implemented yet".into()),
        Command::Info(i) => info(&i),
    }
}
