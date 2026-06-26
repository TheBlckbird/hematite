use std::io::Write;

use hematite_macros::Deserialize;

use crate::protocol::packets::ServerboundPacket;

#[derive(Debug, Deserialize)]
pub struct StatusRequest;

impl ServerboundPacket for StatusRequest {
    fn handle(&self, writer: Box<&mut dyn Write>) {
        todo!()
    }
}
