use crate::protocol::packets::{
    ServerboundPacket,
    all::{ClientboundPackets, Packets, ServerboundPackets},
    handshake::serverbound::Handshake,
};

pub mod serverbound;

pub enum HandshakeSB {
    Handshake(Handshake),
}

impl ServerboundPackets for HandshakeSB {}

impl Packets for HandshakeSB {
    fn get_all() -> Vec<Self> {
        todo!()
    }
}

pub enum HandshakeCB {}

impl ClientboundPackets for HandshakeCB {}

impl Packets for HandshakeCB {
    fn get_all() -> Vec<Self> {
        Vec::new()
    }
}
