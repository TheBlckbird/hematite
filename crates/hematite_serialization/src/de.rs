use std::{
    io::{self, BufRead},
    string::FromUtf8Error,
};

use thiserror::Error;

pub trait Deserialize: Sized {
    fn deserialize<R: BufRead>(reader: &mut R) -> Result<Self, Error>;
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Message(&'static str),
    #[error("Too few bytes; expected {expected}, actual {actual}")]
    TooFewBytes { expected: usize, actual: usize },
    #[error("Syntax error")]
    Snytax,
    #[error("Unsupported type {0}")]
    Unsupported(&'static str),
    #[error("IO Error: {0}")]
    Io(io::Error),
    #[error("UTF8 error: {0}")]
    FromUtf8Error(FromUtf8Error),
    #[error(
        "Too long, expected a maximum length of {expected} items, got {actual} items. Context: {context}"
    )]
    TooLong {
        expected: usize,
        actual: usize,
        context: &'static str,
    },
    #[error("NBT Error: {0}")]
    Nbt(crab_nbt::error::Error),
}
