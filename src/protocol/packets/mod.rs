use hematite_macros::{Deserialize, Packet, Serialize};

use crate::protocol::data_types::{proto_string::ProtoString, var_int::VarInt};

pub mod configuration;
pub mod handshake;
pub mod login;
pub mod play;
pub mod status;

pub trait Packet {
    fn get_id() -> u8;
    fn get_identifier() -> &'static str;
}
