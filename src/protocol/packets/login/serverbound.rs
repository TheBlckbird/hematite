use bevy_ecs::message::Message;
use hematite_macros::Deserialize;
use hematite_serialization::builtin_types::var_int::VarInt;
use uuid::Uuid;

use crate::protocol::data_types::{
    array::Array, identifier::Identifier, proto_string::ProtoString,
};

#[derive(Debug, Deserialize, Message)]
pub struct LoginStart {
    pub name: ProtoString<16>,
    pub uuid: Uuid,
}

#[derive(Debug, Deserialize, Message)]
pub struct LoginPluginResponse {
    pub message_id: VarInt,
    pub data: Option<Vec<u8>>,
}

#[derive(Debug, Deserialize, Message)]
pub struct LoginAcknowledged {}

#[derive(Debug, Deserialize, Message)]
pub struct CookieResponse {
    pub key: Identifier,
    pub payload: Option<Array<u8, 5120>>,
}
