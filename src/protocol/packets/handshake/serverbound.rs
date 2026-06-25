use hematite_macros::{Deserialize, Packet, Serialize};

use crate::protocol::data_types::{proto_string::ProtoString, var_int::VarInt};

#[derive(Packet, Deserialize)]
struct Intention {
    pub protocol_version: VarInt,
    pub server_address: ProtoString<255>,
    pub server_port: u16,
    pub intent: VarInt,
}
