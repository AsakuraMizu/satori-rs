use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use http::{header::AUTHORIZATION, HeaderMap, HeaderValue, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio_tungstenite::{connect_async, tungstenite::client::IntoClientRequest};
use tracing::{debug, error, info, trace};
use url::Host;

use crate::{
    api::{RawApiCall, TypedApiCall},
    error::{ApiError, MapSatoriError, SatoriError},
    structs::{BotId, Channel, ChannelType, Event, Login, Message},
    Satori, SatoriSdk,
};

pub mod events;

type WsMessage = tokio_tungstenite::tungstenite::Message;

pub const ONEBOT: &str = "OneBot";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Onebot11SDKConfig {
    pub host: Host,
    pub ws_port: u16,
    pub http_port: u16,
    pub access_token: Option<String>,
    pub self_id: String,
}

#[derive(Debug)]
pub struct Onebot11SDK {
    config: Onebot11SDKConfig,
    client: reqwest::Client,
}

impl Onebot11SDK {
    pub fn new(config: Onebot11SDKConfig) -> Self {
        let mut headers = HeaderMap::new();
        if let Some(access_token) = &config.access_token {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap(),
            );
        }
        Self {
            config,
            client: reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .unwrap(),
        }
    }

    fn url(&self, api: &str) -> String {
        format!(
            "http://{}:{}/{}",
            self.config.host, self.config.http_port, api
        )
    }
}

impl SatoriSdk for Onebot11SDK {
    async fn start<S>(&self, s: &Arc<S>) -> ()
    where
        S: Satori + Send + Sync + 'static,
    {
        let addr = format!("ws://{}:{}/", self.config.host, self.config.ws_port);
        let mut req = addr.as_str().into_client_request().unwrap();
        if let Some(access_token) = &self.config.access_token {
            req.headers_mut().insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap(),
            );
        }
        let (mut ws_stream, _) = connect_async(req).await.unwrap();
        info!(target: ONEBOT, "WebSocket connected with {addr}");

        loop {
            tokio::select! {
                data = ws_stream.next() => {
                    trace!(target: ONEBOT, "receive ws_msg: {:?}" ,data);
                    match data {
                        Some(Ok(WsMessage::Text(text))) => {
                            debug!(target: ONEBOT, "receive event: {text}");
                            match serde_json::from_str(&text) {
                                Ok(ev) => match ev {
                                    events::Event::Message(msg) => {
                                        let private = msg.message_type == "private";
                                        s.handle_event(Event {
                                            id: msg.message_id as i64,
                                            ty: "message-created".to_string(),
                                            platform: ONEBOT.to_string(),
                                            self_id: msg.self_id.to_string(),
                                            timestamp: msg.time,
                                            channel: Some(Channel {
                                                id: format!("{}:{}", msg.message_type, if private { msg.user_id } else { msg.group_id.unwrap_or_default() }),
                                                ty: Some(ChannelType::Text),
                                                ..Default::default()
                                            }),
                                            message: Some(Message {
                                               id: msg.message_id.to_string(),
                                               content: Some(msg.message),
                                               ..Default::default()
                                            }),
                                            ..Default::default()
                                        }).await;
                                    }
                                    events::Event::Unknown => {}
                                }
                                Err(e) => error!(target: ONEBOT, "failed to parse event: {:?}", e)
                            }
                        }
                        Some(Ok(WsMessage::Ping(d))) => match ws_stream.send(WsMessage::Pong(d)).await {
                            Ok(_) => {}
                            Err(_) => break,
                        }
                        Some(Ok(WsMessage::Pong(_))) => {}
                        _ => break,
                    }
                }
                _ = s.stopped() => {
                    ws_stream.send(WsMessage::Close(None)).await.ok();
                    break;
                }
            }
        }
    }

    async fn call_api<S>(
        &self,
        _s: &Arc<S>,
        bot: &BotId,
        payload: RawApiCall,
    ) -> Result<Value, SatoriError>
    where
        S: Satori + Send + Sync + 'static,
    {
        if !self.has_bot(bot).await {
            return Err(SatoriError::InvalidBot);
        }
        match TypedApiCall::try_from(payload).map_internal_error()? {
            TypedApiCall::MessageCreate {
                channel_id,
                content,
            } => {
                let (ty, id) = channel_id.split_once(":").unwrap();
                match ty {
                    "private" => {
                        let resp = self
                            .client
                            .post(self.url("send_private_msg"))
                            .json(&json!({ "user_id": id, "message": content }))
                            .send()
                            .await
                            .map_internal_error()?;
                        match resp.status() {
                            StatusCode::OK => Ok(resp.json().await.map_internal_error()?),
                            _ => Err(SatoriError::ApiError(ApiError::from_respponse(resp).await?)),
                        }
                    }
                    "group" => {
                        let resp = self
                            .client
                            .post(self.url("send_group_msg"))
                            .json(&json!({ "group_id": id, "message": content }))
                            .send()
                            .await
                            .map_internal_error()?;
                        match resp.status() {
                            StatusCode::OK => Ok(resp.json().await.map_internal_error()?),
                            _ => Err(SatoriError::ApiError(ApiError::from_respponse(resp).await?)),
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    async fn has_bot(&self, bot: &BotId) -> bool {
        bot.platform == ONEBOT && bot.id == self.config.self_id
    }

    async fn get_logins(&self) -> Vec<Login> {
        vec![]
    }
}
