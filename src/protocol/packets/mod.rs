use crate::protocol::packets::{
    handshake::serverbound::Handshake,
    login::{
        clientbound::{CookieRequest, LoginDisconnect, LoginPluginRequest, LoginSuccess},
        serverbound::{CookieResponse, LoginAcknowledged, LoginPluginResponse, LoginStart},
    },
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
    Handshake (networking):
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

    Login (networking):
        SB {
            LoginStart = "hello",
            LoginPluginResponse = "custom_query_answer",
            LoginAcknowledged,
            CookieResponse,
        }
        CB {
            LoginDisconnect,
            LoginSuccess = "login_finished",
            LoginPluginRequest = "custom_query",
            CookieRequest,
        };

    Play:
        SB {}
        CB {};
}
