use hematite_macros::{Deserialize, Serialize};
use uuid::Uuid;

use crate::protocol::data_types::proto_string::ProtoString;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GameProfile {
    pub uuid: Uuid,
    pub username: ProtoString<16>,
    pub properties: Box<[GameProfileProperty]>,
}

impl GameProfile {
    pub fn new(
        uuid: Uuid,
        username: ProtoString<16>,
        properties: Box<[GameProfileProperty]>,
    ) -> Self {
        Self {
            uuid,
            username,
            properties,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GameProfileProperty {
    pub name: ProtoString<64>,
    pub value: ProtoString<32_767>,
    pub signature: Option<ProtoString<1024>>,
}

impl GameProfileProperty {
    pub fn new(
        name: ProtoString<64>,
        value: ProtoString<32_767>,
        signature: Option<ProtoString<1024>>,
    ) -> Self {
        Self {
            name,
            value,
            signature,
        }
    }
}
