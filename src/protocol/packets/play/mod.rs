use crate::protocol::packets::all::{ClientboundPackets, Packets, ServerboundPackets};

pub mod clientbound;
pub mod serverbound;

pub enum PlaySB {}

impl ServerboundPackets for PlaySB {}

impl Packets for PlaySB {
    fn get_all() -> Vec<Self> {
        todo!()
    }
}

pub enum PlayCB {}

impl ClientboundPackets for PlayCB {}

impl Packets for PlayCB {
    fn get_all() -> Vec<Self> {
        todo!()
    }
}
