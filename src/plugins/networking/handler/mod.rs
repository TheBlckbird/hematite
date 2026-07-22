use std::io::{self, Cursor, Read, Write};

use anyhow::Context;
use flume::{Receiver, Sender};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tracing::{error, info};

use crate::{
    plugins::networking::{
        handler::{
            disconnect_client::{disconnect_client_err, disconnect_client_msg},
            handle_login::handle_login_start,
        },
        handshake::handle_handshake,
    },
    protocol::{
        data_types::var_int::VarInt,
        packets::{
            EngineCBPackets, EngineSBPackets, NetworkingSBPackets, RoutedCBPacket, RoutedSBPacket,
            ServerState,
        },
        ser_de::{
            de::{self, Deserialize},
            ser::{self, Serialize},
        },
    },
};

mod bevy_side;
mod disconnect_client;
mod handle_login;

#[derive(Debug)]
struct RawPacket {
    id: u8,
    buffer: Vec<u8>,
}

impl RawPacket {
    fn new(id: u8, buffer: Vec<u8>) -> Self {
        Self { id, buffer }
    }

    async fn send_via_socket(&self, socket: &mut TcpStream) -> anyhow::Result<()> {
        let mut writer = Vec::new();
        self.serialize(&mut writer).context("Error serializing")?;
        socket
            .write_all(&writer)
            .await
            .context("Error writing to socket")?;

        Ok(())
    }
}

impl Serialize for RawPacket {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), ser::Error> {
        let length = VarInt((self.buffer.len() + size_of::<u8>()) as i32);
        length.serialize(writer)?;
        self.id.serialize(writer)?;
        writer.write_all(&self.buffer).map_err(ser::Error::Io)?;

        Ok(())
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
    let mut server_state = ServerState::Handshake;
    let mut player_profile = None;

    loop {
        let header = match get_raw_packet(&mut socket).await {
            Ok(header) => header,
            Err(error) => {
                disconnect_client_err(socket, error).await;
                break;
            }
        };

        let mut reader = Cursor::new(header.buffer);

        let packet = match RoutedSBPacket::from_id(&header.id, &server_state, &mut reader) {
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
            RoutedSBPacket::Networking(packet) => match packet {
                NetworkingSBPackets::Handshake(handshake) => {
                    handle_handshake(handshake, &mut server_state);
                }
                NetworkingSBPackets::LoginStart(login_start) => {
                    player_profile = Some(handle_login_start(login_start, &mut socket).await);
                }
                NetworkingSBPackets::LoginPluginResponse(_login_plugin_response) => todo!(),
                NetworkingSBPackets::LoginAcknowledged(_) => {
                    info!(
                        "Client joined: {}",
                        player_profile
                            .as_ref()
                            .expect("Player Profile should be set in this step")
                    );
                    server_state = ServerState::Configuration;
                }
                NetworkingSBPackets::CookieResponse(_cookie_response) => todo!(),
            },
            RoutedSBPacket::Engine(packet) => handle_engine_packet(packet, &to_bevy_tx).await,
        }
    }
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

fn into_raw_packet(packet: RoutedCBPacket) -> RawPacket {
    let mut writer = Vec::new();
    packet.serialize(&mut writer).unwrap();

    RawPacket::new(packet.get_id(), writer)
}
