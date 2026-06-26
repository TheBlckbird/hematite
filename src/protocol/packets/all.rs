use crate::protocol::packets::{
    configuration::{ConfigurationCB, ConfigurationSB},
    handshake::{HandshakeCB, HandshakeSB},
    login::{LoginCB, LoginSB},
    play::{PlayCB, PlaySB},
    status::{StatusCB, StatusSB},
};

enum PacketsType {
    Handshake(PacketsList<HandshakeSB, HandshakeCB>),
    Status(PacketsList<StatusSB, StatusCB>),
    Login(PacketsList<LoginSB, LoginCB>),
    Configuration(PacketsList<ConfigurationSB, ConfigurationCB>),
    Play(PacketsList<PlaySB, PlayCB>),
}

struct PacketsList<SB: ServerboundPackets, CB: ClientboundPackets> {
    pub clientbound: Vec<CB>,
    pub serverbound: Vec<SB>,
}

impl<SB: ServerboundPackets, CB: ClientboundPackets> PacketsList<SB, CB> {
    pub fn get_all() -> Self {
        Self {
            clientbound: CB::get_all(),
            serverbound: SB::get_all(),
        }
    }
}

pub trait Packets: Sized {
    fn get_all() -> Vec<Self>;
}

pub trait ClientboundPackets: Packets {}
pub trait ServerboundPackets: Packets {}
