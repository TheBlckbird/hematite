use hematite_macros::Serialize;

use crate::protocol::data_types::proto_string::ProtoString;

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub json_response: ProtoString,
}
