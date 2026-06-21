use std::fmt::Display;

use serde::{de, ser};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Message(String),
    #[error("Syntax error")]
    Snytax,
    #[error("Unsupported type {0}")]
    Unsupported(&'static str),
    // More to follow, just an example
}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::Message(msg.to_string())
    }
}
