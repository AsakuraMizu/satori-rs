use std::{future::Future, sync::Arc};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use crate::{
    error::{MapSatoriError, SatoriError},
    structs::*,
    Satori,
};

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct RawApiCall {
    pub method: String,
    pub body: Value,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(tag = "method", content = "body")]
pub enum TypedApiCall {
    #[serde(rename = "message.create")]
    MessageCreate { channel_id: String, content: String },
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
impl IntoRawApiCall for TypedApiCall {
    fn into_raw(self) -> RawApiCall {
        // this never fails
        serde_json::from_value(serde_json::to_value(self).unwrap()).unwrap()
    }
}

impl TryFrom<RawApiCall> for TypedApiCall {
    type Error = serde_json::Error;

    fn try_from(value: RawApiCall) -> Result<Self, Self::Error> {
        serde_json::from_value(serde_json::to_value(value).unwrap())
    }
}

pub trait SatoriApi: Satori + sealed::Sealed {
    fn call_api_typed<R>(
        self: &Arc<Self>,
        bot: &BotId,
        payload: TypedApiCall,
    ) -> impl Future<Output = Result<R, SatoriError>> + Send
    where
        R: DeserializeOwned;

    fn create_message(
        self: &Arc<Self>,
        bot: &BotId,
        channel_id: String,
        content: String,
    ) -> impl Future<Output = Result<Value, SatoriError>> + Send;
}

impl<S> SatoriApi for S
where
    S: Satori + Send + Sync,
{
    async fn call_api_typed<R>(
        self: &Arc<Self>,
        bot: &BotId,
        payload: TypedApiCall,
    ) -> Result<R, SatoriError>
    where
        R: DeserializeOwned,
    {
        Ok(serde_json::from_value(self.call_api(bot, payload).await?).map_internal_error()?)
    }

    async fn create_message(
        self: &Arc<Self>,
        bot: &BotId,
        channel_id: String,
        content: String,
    ) -> Result<Value, SatoriError> {
        self.call_api_typed(
            bot,
            TypedApiCall::MessageCreate {
                channel_id,
                content,
            },
        )
        .await
    }
}

mod sealed {
    pub trait Sealed {}
    impl<S> Sealed for S where S: crate::Satori {}
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{IntoRawApiCall, RawApiCall, TypedApiCall};

    #[test]
    fn test_convert() {
        assert_eq!(
            TypedApiCall::MessageCreate {
                channel_id: "1".to_string(),
                content: "1".to_string(),
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
                channel_id: "2".to_string(),
                content: "2".to_string(),
            }
        );

        assert!(TypedApiCall::try_from(RawApiCall {
            method: "wtf".to_string(),
            body: json!(null),
        })
        .is_err());
    }
}
