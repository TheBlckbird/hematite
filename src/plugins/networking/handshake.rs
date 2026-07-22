use crate::protocol::packets::{
    ServerState,
    handshake::serverbound::{Handshake, Intent},
};

/// Set the server_state to the state set by the handshake
pub fn handle_handshake(handshake: Handshake, server_state: &mut ServerState) {
    match handshake.intent {
        Intent::Status => *server_state = ServerState::Status,
        Intent::Login => *server_state = ServerState::Login,
        Intent::Transfer => *server_state = ServerState::Login,
    }
}
