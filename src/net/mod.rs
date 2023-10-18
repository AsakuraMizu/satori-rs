pub mod app;
pub mod sdk;

use serde::{Deserialize, Serialize};

use crate::{Event, Login};

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum Signal {
    Event { op: OpCode<0>, body: Event },
    Ping { op: OpCode<1>, body: Option<Empty> },
    Pong { op: OpCode<2>, body: Option<Empty> },
    Identify { op: OpCode<3>, body: Identify },
    Ready { op: OpCode<4>, body: Logins },
}

#[derive(Debug, Deserialize, Serialize)]
struct Empty {}

#[derive(Debug, Deserialize, Serialize)]
struct Logins {
    logins: Vec<Login>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Identify {
    pub token: Option<String>,
    pub sequence: Option<i64>,
}

// source: https://github.com/serde-rs/serde/issues/745#issuecomment-1450072069
#[derive(Debug)]
struct OpCode<const V: u8>;

impl<const V: u8> Serialize for OpCode<V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(V)
    }
}

impl<'de, const V: u8> Deserialize<'de> for OpCode<V> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        if value == V {
            Ok(OpCode::<V>)
        } else {
            Err(serde::de::Error::custom("invalid signal type"))
        }
    }
}

impl Signal {
    fn event(event: Event) -> Self {
        Self::Event {
            op: OpCode,
            body: event,
        }
    }

    fn ping() -> Self {
        Self::Ping {
            op: OpCode,
            body: None,
        }
    }

    fn pong() -> Self {
        Self::Pong {
            op: OpCode,
            body: None,
        }
    }
    fn identify(token: &str, seq: i64) -> Self {
        Self::Identify {
            op: OpCode,
            body: Identify {
                token: Some(token.to_string()),
                sequence: Some(seq),
            },
        }
    }
    fn ready(logins: Vec<Login>) -> Self {
        Self::Ready {
            op: OpCode,
            body: Logins { logins },
        }
    }
}

impl ToString for Signal {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}
