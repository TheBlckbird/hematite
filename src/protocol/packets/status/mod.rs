use crate::protocol::packets::{
    all::{ClientboundPackets, Packets, ServerboundPackets},
    status::{clientbound::ping_response::PongResponse, serverbound::ping_request::PingRequest},
};

pub mod clientbound;
pub mod serverbound;

pub enum StatusSB {
    PingRequest(PingRequest),
}

impl ServerboundPackets for StatusSB {}

impl Packets for StatusSB {
    fn get_all() -> Vec<Self> {
        todo!()
    }
}

pub enum StatusCB {
    PongResponse(PongResponse),
}

impl ClientboundPackets for StatusCB {}

impl Packets for StatusCB {
    fn get_all() -> Vec<Self> {
        todo!()
    }
}
