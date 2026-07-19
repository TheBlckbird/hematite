use hematite_ecs::prelude::*;
use hematite_macros::Deserialize;

#[derive(Debug, Deserialize, Message)]
pub struct StatusRequest;
