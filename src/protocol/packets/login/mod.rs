use crate::protocol::packets::all::{ClientboundPackets, Packets, ServerboundPackets};

pub mod clientbound;
pub mod serverbound;

pub enum LoginSB {}

impl ServerboundPackets for LoginSB {}

impl Packets for LoginSB {
    fn get_all() -> Vec<Self> {
        todo!()
    }
}

pub enum LoginCB {}

impl ClientboundPackets for LoginCB {}

impl Packets for LoginCB {
    fn get_all() -> Vec<Self> {
        todo!()
    }
}
