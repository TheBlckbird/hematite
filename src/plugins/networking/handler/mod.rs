use std::io::{self, Cursor, Read};

use flume::{Receiver, Sender};
use thiserror::Error;
use tokio::{io::AsyncReadExt, net::TcpStream};
use tracing::error;

use crate::{
    plugins::networking::handler::disconnect_client::{
        disconnect_client_err, disconnect_client_msg,
    },
    protocol::{
        data_types::var_int::VarInt,
        packets::{
            EngineCBPackets, EngineSBPackets, NetworkingSBPackets, RoutedSBPacket, ServerState,
        },
        ser_de::de::{self, Deserialize},
    },
};

mod bevy_side;
mod disconnect_client;
mod handle_handshake;
mod handle_login;

struct RawPacket {
    id: u8,
    buffer: Vec<u8>,
}

impl RawPacket {
    fn new(id: u8, buffer: Vec<u8>) -> Self {
        Self { id, buffer }
    }
}

#[derive(Debug, Error)]
enum PacketError {
    #[error("{0}")]
    Deserialization(de::Error),
    #[error("{0}")]
    Io(io::Error),
}

pub async fn handle_client(
    mut socket: TcpStream,
    to_bevy_tx: Sender<EngineSBPackets>,
    _to_networking_rx: Receiver<EngineCBPackets>,
) {
    let current_state = ServerState::Handshake;

    loop {
        let header = match get_raw_packet(&mut socket).await {
            Ok(header) => header,
            Err(error) => {
                disconnect_client_err(socket, error).await;
                break;
            }
        };

        let mut reader = Cursor::new(header.buffer);

        let packet = match RoutedSBPacket::from_id(&header.id, &current_state, &mut reader) {
            Some(Ok(packet)) => packet,
            None => {
                disconnect_client_msg(socket, format!("Unknown id {}", header.id)).await;
                break;
            }
            Some(Err(error)) => {
                disconnect_client_err(socket, error).await;
                break;
            }
        };

        match packet {
            RoutedSBPacket::Networking(packet) => handle_networking_packet(packet).await,
            RoutedSBPacket::Engine(packet) => handle_engine_packet(packet, &to_bevy_tx).await,
        }
    }
}

async fn handle_networking_packet(packet: NetworkingSBPackets) {
    todo!()
}

async fn handle_engine_packet(packet: EngineSBPackets, to_bevy_tx: &Sender<EngineSBPackets>) {
    let transmit_result = to_bevy_tx.send_async(packet).await;

    if let Err(error) = transmit_result {
        error!("Couldn't transmit packet to bevy {error}");
    }
}

async fn get_raw_packet(socket: &mut TcpStream) -> Result<RawPacket, PacketError> {
    let length = *VarInt::from_socket(socket)
        .await
        .map_err(PacketError::Deserialization)? as usize;

    // Create a buffer with the given length and write the following n bytes into it
    let mut buffer = vec![0u8; length];

    socket
        .read_exact(&mut buffer)
        .await
        .map_err(PacketError::Io)?;

    let mut reader = Cursor::new(buffer);
    let id = *VarInt::deserialize(&mut reader).map_err(PacketError::Deserialization)? as u8;

    let mut remaining = Vec::new();
    Read::read_to_end(&mut reader, &mut remaining).map_err(PacketError::Io)?;

    Ok(RawPacket::new(id, remaining))
}
