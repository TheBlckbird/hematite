use crate::protocol::packets::all::{ClientboundPackets, Packets, ServerboundPackets};

pub mod clientbound;
pub mod serverbound;

pub enum ConfigurationSB {}

impl ServerboundPackets for ConfigurationSB {}

impl Packets for ConfigurationSB {
    fn get_all() -> Vec<Self> {
        todo!()
    }
}

pub enum ConfigurationCB {}

impl ClientboundPackets for ConfigurationCB {}

impl Packets for ConfigurationCB {
    fn get_all() -> Vec<Self> {
        todo!()
    }
}
