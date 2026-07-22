use hematite_macros::Serialize;

use crate::protocol::data_types::{game_profile::GameProfile, proto_string::ProtoString};

#[derive(Debug, Serialize)]
pub struct LoginDisconnect {
    pub reason: ProtoString, // [TODO] string for now, needs to be a JSON text component later
}

#[derive(Debug, Serialize)]
pub struct LoginSuccess {
    pub profile: GameProfile,
}

#[derive(Debug, Serialize)]
pub struct LoginPluginRequest {}

#[derive(Debug, Serialize)]
pub struct CookieRequest {}
