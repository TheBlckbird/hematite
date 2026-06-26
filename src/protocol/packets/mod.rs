use std::{
    fmt::Debug,
    io::{BufRead, Write},
};

use crate::{
    PacketError,
    protocol::{
        data_types::var_int::VarInt,
        ser_de::de::{self, Deserialize},
    },
};

mod all;
pub mod configuration;
pub mod handshake;
pub mod login;
pub mod play;
pub mod status;

#[derive(Debug)]
pub struct PacketHeader {
    pub id: u8,
    pub data: Vec<u8>,
}

impl Deserialize for PacketHeader {
    fn deserialize<R: BufRead>(reader: &mut R) -> Result<Self, de::Error> {
        let length = *VarInt::deserialize(reader)?;
        let packet_id = VarInt::deserialize(reader)?;
        let data_length = length as usize - packet_id.len();

        let mut data = vec![0; data_length];
        reader.read_exact(&mut data).map_err(de::Error::Io)?;

        Ok(Self {
            id: packet_id.into_inner() as u8,
            data,
        })
    }
}

pub trait ServerboundPacket: Debug {
    fn handle(&self, writer: Box<&mut dyn Write>);
}

pub trait ClientboundPacket: Debug {}
