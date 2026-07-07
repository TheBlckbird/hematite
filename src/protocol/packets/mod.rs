use crate::protocol::packets::{
    handshake::serverbound::Handshake,
    status::{
        clientbound::{ping_response::PongResponse, status_response::StatusResponse},
        serverbound::{ping_request::PingRequest, status_request::StatusRequest},
    },
};

use hematite_macros::all_packets;

pub mod configuration;
pub mod handshake;
pub mod login;
pub mod play;
pub mod status;

all_packets! {
    Handshake:
        SB {
            Handshake = "intention",
        };

    Status:
        SB {
            PingRequest,
            StatusRequest,
        }
        CB {
            PongResponse,
            StatusResponse,
        };

    Configuration:
        SB {}
        CB {};

    Login:
        SB {}
        CB {};

    Play:
        SB {}
        CB {};
}
