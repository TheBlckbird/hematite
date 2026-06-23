use std::{
    io::{BufRead, Write},
    ops::{Deref, DerefMut},
};

use crate::protocol::{
    data_types::proto_string::ProtoString,
    ser_de::{
        de::{self, Deserialize},
        ser::{self, Serialize},
    },
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identifier(ProtoString);

impl Identifier {
    pub fn new(identifier: impl Into<String>) -> Self {
        Self(identifier.into().into())
    }

    pub fn into_inner(self) -> String {
        self.0.into_inner()
    }
}

impl Deref for Identifier {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Identifier {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Serialize for Identifier {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), ser::Error> {
        self.0.serialize(writer)
    }
}

impl Deserialize for Identifier {
    fn deserialize<R: BufRead>(reader: &mut R) -> Result<Self, de::Error> {
        ProtoString::deserialize(reader).map(Identifier)
    }
}
