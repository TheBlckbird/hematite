use hematite_macros::Deserialize;

use crate::protocol::data_types::proto_string::ProtoString;

#[derive(Debug, Deserialize)]
pub struct StatusResponse {
    pub json_response: ProtoString,
}
