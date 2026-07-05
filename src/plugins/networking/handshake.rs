use hematite_ecs::prelude::*;
use tracing::debug;

use crate::protocol::packets::handshake::serverbound::Handshake;

pub fn handle_handshake(handshake: On<Handshake>) {
    debug!("Handshake {handshake:#?}");
}
