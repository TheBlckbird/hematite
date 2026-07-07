use hematite_ecs::prelude::*;
use hematite_macros::Deserialize;

use crate::protocol::packets::ServerboundPacket;

#[derive(Debug, Deserialize, Message)]
pub struct StatusRequest;

impl ServerboundPacket for StatusRequest {}
