use std::sync::Arc;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use crate::{error::SatoriError, structs::*, AppT, Satori, SdkT};

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct RawApiCall {
    pub method: String,
    pub body: Value,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(tag = "api", content = "body")]
pub enum TypedApiCall<'a> {
    #[serde(rename = "message.create")]
    MessageCreate {
        channel_id: &'a str,
        content: &'a str,
    },
}

pub trait IntoRawApiCall {
    fn into_raw(self) -> RawApiCall;
}

impl IntoRawApiCall for RawApiCall {
    fn into_raw(self) -> RawApiCall {
        self
    }
}

// TODO: rewrite this part
impl<'a> IntoRawApiCall for TypedApiCall<'a> {
    fn into_raw(self) -> RawApiCall {
        // this never fails
        serde_json::from_value(serde_json::to_value(self).unwrap()).unwrap()
    }
}

impl<'a> TryFrom<RawApiCall> for TypedApiCall<'a> {
    type Error = serde_json::Error;

    fn try_from(value: RawApiCall) -> Result<Self, Self::Error> {
        Self::deserialize(serde_json::to_value(value).unwrap())
    }
}

impl<S, A> Satori<S, A>
where
    S: SdkT + Send + Sync + 'static,
    A: AppT + Send + Sync + 'static,
{
    async fn _call_api<R>(
        self: &Arc<Self>,
        bot: &BotId,
        payload: TypedApiCall<'_>,
    ) -> Result<R, SatoriError>
    where
        R: DeserializeOwned,
    {
        Ok(serde_json::from_str(&self.call_api(bot, payload).await?)
            .map_err(anyhow::Error::from)?)
    }

    pub async fn create_message(
        self: &Arc<Self>,
        bot: &BotId,
        channel_id: &str,
        content: &str,
    ) -> Result<Vec<Message>, SatoriError> {
        self._call_api(
            bot,
            TypedApiCall::MessageCreate {
                channel_id,
                content,
            },
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{IntoRawApiCall, RawApiCall, TypedApiCall};

    #[test]
    fn test_convert() {
        assert_eq!(
            TypedApiCall::MessageCreate {
                channel_id: "1",
                content: "1",
            }
            .into_raw(),
            RawApiCall {
                method: "message.create".to_string(),
                body: json!({ "channel_id": "1", "content": "1" })
            }
        );

        assert_eq!(
            TypedApiCall::try_from(RawApiCall {
                method: "message.create".to_string(),
                body: json!({ "channel_id": "2", "content": "2" }),
            })
            .unwrap(),
            TypedApiCall::MessageCreate {
                channel_id: "2",
                content: "2",
            }
        );

        assert!(TypedApiCall::try_from(RawApiCall {
            method: "wtf".to_string(),
            body: json!(null),
        })
        .is_err());
    }
}
