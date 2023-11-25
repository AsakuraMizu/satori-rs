use std::{collections::HashMap, sync::Arc};

use futures_util::{SinkExt, StreamExt};
use http::{header::AUTHORIZATION, HeaderValue};
use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::{mpsc, oneshot, Mutex},
};
use tokio_tungstenite::{connect_async, tungstenite::client::IntoClientRequest, WebSocketStream};
use tracing::{debug, error, info, trace, warn};

use crate::{
    api::{RawApiCall, TypedApiCall},
    error::{MapSatoriError, SatoriError},
    structs::{BotId, Channel, ChannelType, Event, Login, Message},
    Satori, SatoriSDK,
};

pub mod structs;

type WsMessage = tokio_tungstenite::tungstenite::Message;
type WsError = tokio_tungstenite::tungstenite::Error;

pub const ONEBOT: &str = "OneBot";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Onebot11SDKConfig {
    pub host: String,
    pub port: u16,
    pub access_token: Option<String>,
    pub self_id: String,
}

type ActionCallbackPayload = Result<structs::ActionResp, SatoriError>;

type ActionPayload = (structs::Action, oneshot::Sender<ActionCallbackPayload>);

#[derive(Debug)]
pub struct Onebot11SDK {
    config: Onebot11SDKConfig,
    action_tx: mpsc::Sender<ActionPayload>,
    action_rx: Arc<Mutex<mpsc::Receiver<ActionPayload>>>,
}

impl Onebot11SDK {
    pub fn new(config: Onebot11SDKConfig) -> Self {
        let (action_tx, action_rx) = mpsc::channel(100);
        Self {
            config,
            action_tx,
            action_rx: Arc::new(Mutex::new(action_rx)),
        }
    }

    async fn handle_ws_msg<S, T>(
        msg: Option<Result<WsMessage, WsError>>,
        s: &Arc<S>,
        ws_stream: &mut WebSocketStream<T>,
        action_resp_map: &mut HashMap<String, oneshot::Sender<ActionCallbackPayload>>,
    ) -> bool
    where
        S: Satori + Send + Sync + 'static,
        T: AsyncRead + AsyncWrite + Unpin,
    {
        trace!(target: ONEBOT, "receive ws_msg: {:?}" ,msg);
        match msg {
            Some(Ok(WsMessage::Text(text))) => {
                debug!(target: ONEBOT, "receive event: {text}");
                match serde_json::from_str(&text) {
                    Ok(structs::EventOrActionResp::Event(ev)) => {
                        if let Some(ev) = Onebot11SDK::transform_event(ev) {
                            s.handle_event(ev);
                        }
                    }
                    Ok(structs::EventOrActionResp::ActionResp(resp)) => {
                        let Some(echo) = resp.echo.as_deref() else {
                            warn!(target: ONEBOT, "action response missing echo, ignoring");
                            return true;
                        };
                        let Some(tx) = action_resp_map.remove(echo) else {
                            warn!(target: ONEBOT, "action caller not found, ignoring");
                            return true;
                        };
                        let _ = tx.send(Ok(resp));
                    }
                    Err(e) => error!(target: ONEBOT, "failed to parse event: {:?}", e),
                }
                true
            }
            Some(Ok(WsMessage::Ping(d))) => ws_stream.send(WsMessage::Pong(d)).await.is_ok(),
            Some(Ok(WsMessage::Pong(_))) => true,
            _ => false,
        }
    }

    fn transform_event(ev: structs::Event) -> Option<Event> {
        match ev {
            structs::Event::Message(msg) => {
                let private = msg.message_type == "private";
                Some(Event {
                    id: msg.message_id as i64,
                    ty: "message-created".to_string(),
                    platform: ONEBOT.to_string(),
                    self_id: msg.self_id.to_string(),
                    timestamp: msg.time,
                    channel: Some(Channel {
                        id: format!(
                            "{}:{}",
                            msg.message_type,
                            if private {
                                msg.user_id
                            } else {
                                msg.group_id.unwrap_or_default()
                            }
                        ),
                        ty: Some(ChannelType::Text),
                        ..Default::default()
                    }),
                    message: Some(Message {
                        id: msg.message_id.to_string(),
                        content: Some(msg.message),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            }
            structs::Event::Unknown => None,
        }
    }
}

impl SatoriSDK for Onebot11SDK {
    async fn start<S>(&self, s: &Arc<S>)
    where
        S: Satori + Send + Sync + 'static,
    {
        let addr = format!("ws://{}:{}/", self.config.host, self.config.port);
        let mut req = addr.as_str().into_client_request().unwrap();
        if let Some(access_token) = &self.config.access_token {
            req.headers_mut().insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap(),
            );
        }
        let (mut ws_stream, _) = connect_async(req).await.unwrap();
        info!(target: ONEBOT, "WebSocket connected with {addr}");

        let mut action_rx = self.action_rx.lock().await;
        let mut action_resp_map = HashMap::<String, oneshot::Sender<ActionCallbackPayload>>::new();

        loop {
            tokio::select! {
                action = action_rx.recv() => if let Some((action, tx)) = action {
                    if let Some(echo) = action.echo.clone() {
                        action_resp_map.insert(echo.clone(), tx);
                    }
                    if let Err(e) = ws_stream.send(WsMessage::text(serde_json::to_string(&action).unwrap())).await {
                        if let Some(echo) = action.echo.as_deref() {
                            if let Some(tx) = action_resp_map.remove(echo) {
                                let _ = tx.send(Err(SatoriError::InternalError(e.into())));
                            }
                        }
                    }
                },
                msg = ws_stream.next() => if !Onebot11SDK::handle_ws_msg(msg, s, &mut ws_stream, &mut action_resp_map).await { break },
                _ = s.stopped() => {
                    let _ = ws_stream.send(WsMessage::Close(None)).await;
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
        let (action, params) = match TypedApiCall::try_from(payload).map_internal_error()? {
            TypedApiCall::MessageCreate {
                channel_id,
                content,
            } => {
                let (ty, id) = channel_id.split_once(":").unwrap();
                match ty {
                    "private" => (
                        "send_private_msg",
                        json!({ "user_id": id, "message": content }),
                    ),
                    "group" => (
                        "send_group_msg",
                        json!({ "group_id": id, "message": content }),
                    ),
                    _ => unreachable!(),
                }
            }
        };
        let echo = Alphanumeric.sample_string(&mut thread_rng(), 8);
        let action = structs::Action {
            action: action.to_string(),
            params,
            echo: Some(echo),
        };
        let (tx, rx) = oneshot::channel();
        self.action_tx
            .clone()
            .send((action, tx))
            .await
            .map_internal_error()?;
        let resp = rx.await.map_internal_error()??;
        Ok(resp.data)
    }

    async fn has_bot(&self, bot: &BotId) -> bool {
        bot.platform == ONEBOT && bot.id == self.config.self_id
    }

    async fn get_logins(&self) -> Vec<Login> {
        vec![]
    }
}
