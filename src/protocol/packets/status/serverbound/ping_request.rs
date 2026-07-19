use hematite_ecs::prelude::*;
use hematite_macros::Deserialize;

#[derive(Debug, Deserialize, Message)]
pub struct PingRequest {
    pub timestamp: i64,
}
