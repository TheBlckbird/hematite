use std::io::{self, Write};

use thiserror::Error;

pub trait Serialize {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error>;
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(
        "Too long, expected a maximum length of {expected} items, got {actual} items. Context: {context}"
    )]
    TooLong {
        expected: usize,
        actual: usize,
        context: &'static str,
    },
    #[error("IO Error: {0}")]
    Io(io::Error),
    #[error("NBT Error: {0}")]
    Nbt(crab_nbt::error::Error),
}
