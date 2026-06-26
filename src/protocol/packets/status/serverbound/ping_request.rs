use std::io::Write;

use hematite_macros::Deserialize;

use crate::protocol::packets::ServerboundPacket;

#[derive(Debug, Deserialize)]
pub struct PingRequest {
    pub timestamp: i64,
}

impl ServerboundPacket for PingRequest {
    fn handle(&self, writer: Box<&mut dyn Write>) {
        todo!()
    }
}
