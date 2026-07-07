use std::collections::HashMap;

use serde::Deserialize;

pub type Packets = HashMap<String, Sides>;

#[derive(Deserialize)]
pub struct Sides {
    pub clientbound: Option<HashMap<String, Packet>>,
    pub serverbound: Option<HashMap<String, Packet>>,
}

#[derive(Deserialize)]
pub struct Packet {
    pub protocol_id: u8,
}
