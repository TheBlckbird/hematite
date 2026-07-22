use std::io::BufRead;

use hematite_ecs::prelude::*;
use hematite_macros::Deserialize;
use hematite_serialization::{
    builtin_types::var_int::VarInt,
    de::{self, Deserialize},
};

use crate::protocol::data_types::proto_string::ProtoString;

#[derive(Debug, Deserialize, Message)]
pub struct Handshake {
    pub protocol_version: VarInt,
    pub server_address: ProtoString<255>,
    pub server_port: u16,
    pub intent: Intent,
}

#[derive(Debug, Hash)]
pub enum Intent {
    Status,
    Login,
    Transfer,
}

impl Deserialize for Intent {
    fn deserialize<R: BufRead>(reader: &mut R) -> Result<Self, de::Error> {
        let intent = *VarInt::deserialize(reader)?;

        match intent {
            1 => Ok(Self::Status),
            2 => Ok(Self::Login),
            3 => Ok(Self::Transfer),
            _ => Err(de::Error::Syntax),
        }
    }
}
