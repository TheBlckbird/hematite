use std::{
    hash::Hash,
    io::{BufRead, Cursor},
};

use bevy_ecs::system::BoxedSystem;
use hematite_ecs::prelude::*;

use crate::protocol::{
    packets::{
        handshake::serverbound::Handshake,
        status::serverbound::{ping_request::PingRequest, status_request::StatusRequest},
    },
    ser_de::de::{self, Deserialize},
};

pub mod configuration;
pub mod handshake;
pub mod login;
pub mod play;
pub mod status;

pub trait ServerboundPacket: Event {}

pub trait ClientboundPacket {}

pub enum ServerState {
    Handshake,
    Configuration,
    Status,
    Login,
    Play,
}

pub enum Direction {
    Serverbound,
    Clientbound,
}

/*
all_packets! {
    Handshake:
        SB {
            Handshake = "intention",
        }

    Status:
        SB {
            PingRequest,
            StatusRequest,
        }
        CB {
            PongResponse,
            StatusResponse,
        }

    Configuration:
        SB {}
        CB {}

    Login:
        SB {}
        CB {}

    Play:
        SB {}
        CB {}
}
*/

// should result in:

pub enum AllSBPackets {
    Handshake(Handshake),
    PingRequest(PingRequest),
    StatusRequest(StatusRequest),
    // PongResponse(PongResponse),
    // StatusResponse(StatusResponse),
}

impl AllSBPackets {
    pub fn from_id<R: BufRead>(
        id: &u8,
        server_state: &ServerState,
        reader: &mut R,
    ) -> Option<Result<Self, de::Error>> {
        match (id, server_state) {
            (0x00, ServerState::Handshake) => {
                Some(Handshake::deserialize(reader).map(|packet| Self::Handshake(packet)))
            }
            (0x00, ServerState::Status) => {
                Some(StatusRequest::deserialize(reader).map(|packet| Self::StatusRequest(packet)))
            }
            (0x01, ServerState::Status) => {
                Some(PingRequest::deserialize(reader).map(|packet| Self::PingRequest(packet)))
            }
            _ => None,
        }
    }

    pub fn send_event(self, mut commands: Commands) {
        match self {
            AllSBPackets::Handshake(handshake) => commands.trigger(handshake),
            AllSBPackets::PingRequest(ping_request) => commands.trigger(ping_request),
            AllSBPackets::StatusRequest(status_request) => commands.trigger(status_request),
        }
    }
}

pub enum AllCBPackets {}

fn read_packets_channel(mut commands: Commands) {
    let buffer = Vec::new();
    let mut reader = Cursor::new(buffer);
    let packet = AllSBPackets::from_id(&1, &ServerState::Configuration, &mut reader)
        .unwrap()
        .unwrap();

    packet.send_event(commands.reborrow());
}
