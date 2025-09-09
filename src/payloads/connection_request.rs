use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ConnectionRequest {
    #[serde(rename = "start_connection")]
    StartConnection { token: String },

    #[serde(rename = "disconnect")]
    Disconnect { session_id: String, user_id: u64 },
}