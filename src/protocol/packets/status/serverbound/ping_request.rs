use hematite_ecs::prelude::*;
use hematite_macros::Deserialize;

use crate::protocol::packets::ServerboundPacket;

#[derive(Debug, Deserialize, Event)]
pub struct PingRequest {
    pub timestamp: i64,
}

impl ServerboundPacket for PingRequest {}
