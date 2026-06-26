use std::io::{BufRead, BufReader};

use derive_more::{Deref, DerefMut};
use hematite_ecs::prelude::*;
use thiserror::Error;

use crate::protocol::{
    packets::{
        PacketHeader, ServerboundPacket,
        handshake::serverbound::Handshake,
        status::serverbound::{ping_request::PingRequest, status_request::StatusRequest},
    },
    ser_de::de::{self, Deserialize},
};

mod protocol;

const PORT: u16 = 25565;

enum ServerState {
    Handshake,
    Status,
    Login,
    Configuration,
    Play,
}

#[derive(Debug, Error)]
enum PacketError {
    #[error("{0}")]
    Deserialization(de::Error),
    #[error("Unknown packet id {0}")]
    UnknownId(u8),
}

#[derive(Resource, Default, Deref, DerefMut)]
struct Counter(u8);

fn main() {
    let _ = App::new()
        .init_resource::<Counter>()
        .add_systems(TickUpdate, main)
        .run();

    // let address = SocketAddr::from(([0, 0, 0, 0], PORT));
    // let listener = TcpListener::bind(address).unwrap();

    // let current_state = ServerState::Handshake;

    // for stream in listener.incoming() {
    //     let Ok(mut stream) = stream else {
    //         eprintln!("Connection failed");
    //         exit(1);
    //     };
    //     let mut reader = BufReader::new(&stream);
    //     let Ok(packet) = get_packet_from_id(&current_state, &mut reader) else {
    //         stream
    //             .shutdown(Shutdown::Both)
    //             .expect("Shutting down stream failed");
    //         break;
    //     };

    //     packet.handle(Box::new(&mut stream));
    // }
}

fn get_packet_from_id<R: BufRead>(
    current_state: &ServerState,
    reader: &mut R,
) -> Result<Box<dyn ServerboundPacket>, PacketError> {
    let packet = PacketHeader::deserialize(reader).map_err(PacketError::Deserialization)?;
    let mut reader = BufReader::new(packet.data.as_slice());

    match current_state {
        ServerState::Handshake => match packet.id {
            0x00 => Handshake::deserialize(&mut reader)
                .map(|p| -> Box<dyn ServerboundPacket> { Box::new(p) })
                .map_err(PacketError::Deserialization),
            _ => Err(PacketError::UnknownId(packet.id)),
        },
        ServerState::Status => match packet.id {
            0x00 => StatusRequest::deserialize(&mut reader)
                .map(|p| -> Box<dyn ServerboundPacket> { Box::new(p) })
                .map_err(PacketError::Deserialization),

            0x01 => PingRequest::deserialize(&mut reader)
                .map(|p| -> Box<dyn ServerboundPacket> { Box::new(p) })
                .map_err(PacketError::Deserialization),

            _ => Err(PacketError::UnknownId(packet.id)),
        },
        ServerState::Login => todo!(),
        ServerState::Configuration => todo!(),
        ServerState::Play => todo!(),
    }
}
