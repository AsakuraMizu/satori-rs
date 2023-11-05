use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BotId {
    pub id: String,
    pub platform: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Event {
    pub id: i64,
    #[serde(rename = "type")]
    pub ty: String,
    pub platform: String,
    pub self_id: String,
    pub timestamp: i64,
    pub channel: Option<Channel>,
    pub guild: Option<Guild>,
    pub login: Option<Login>,
    pub message: Option<Message>,
    pub member: Option<GuildMember>,
    pub operator: Option<User>,
    pub role: Option<GuildRole>,
    pub user: Option<User>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Channel {
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub ty: Option<ChannelType>,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum ChannelType {
    Text = 0,
    Voice = 1,
    Category = 2,
    Direct = 3,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Guild {
    pub id: String,
    pub name: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Login {
    pub user: Option<User>,
    pub self_id: Option<String>,
    pub platform: Option<String>,
    pub status: Status,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    pub name: Option<String>,
    pub nick: Option<String>,
    pub avatar: Option<String>,
    pub is_bot: Option<bool>,
}

#[derive(Debug, Clone, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum Status {
    Offline = 0,
    Online = 1,
    Connect = 2,
    Disconnect = 3,
    Reconnect = 4,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GuildMember {
    pub user: Option<User>,
    pub nick: Option<String>,
    pub avatar: Option<String>,
    pub joined_at: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GuildRole {
    pub id: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    pub id: String,
    pub content: Option<String>,
    pub channel: Option<Channel>,
    pub guild: Option<Guild>,
    pub member: Option<GuildMember>,
    pub user: Option<User>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Pagination<T> {
    pub data: Vec<T>,
    pub next: String,
}
