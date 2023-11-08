use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Message {
    pub time: i64,
    pub self_id: i64,
    pub post_type: String,
    pub message_type: String,
    pub sub_type: String,
    pub message_id: i32,
    pub user_id: i64,
    pub message: String,
    pub raw_message: String,
    pub font: i32,
    pub target_id: Option<i64>,
    pub group_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "post_type")]
pub enum Event {
    #[serde(rename = "message")]
    Message(Message),
    #[serde(other)]
    Unknown,
}
