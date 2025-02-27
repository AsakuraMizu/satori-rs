use std::{collections::HashSet, net::Ipv4Addr, sync::Arc, time::Duration};

use futures_util::{SinkExt, StreamExt};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{sync::RwLock, time::Instant};
use tokio_tungstenite::connect_async;
use tracing::{error, info, trace};

use super::{Logins, Signal};
use crate::{
    api::RawApiCall,
    error::{ApiError, MapSatoriError, SatoriError},
    impls::net::NET,
    structs::{BotId, Login, Status},
    Satori, SatoriSDK,
};

type WsMessage = tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetSDKConfig {
    pub host: String,
    pub port: u16,
    pub path: Option<String>,
    pub token: Option<String>,
}

impl Default for NetSDKConfig {
    fn default() -> Self {
        Self {
            host: Ipv4Addr::LOCALHOST.to_string(),
            port: 5140,
            path: None,
            token: None,
        }
    }
}

#[derive(Debug)]
pub struct NetSDK {
    config: NetSDKConfig,
    pub bots: Arc<RwLock<HashSet<BotId>>>,
    client: reqwest::Client,
}

impl NetSDK {
    pub fn new(config: NetSDKConfig) -> Self {
        Self {
            config,
            bots: Default::default(),
            client: reqwest::Client::new(),
        }
    }
}

impl SatoriSDK for NetSDK {
    #[allow(unused_assignments)]
    // TODO: seq
    async fn start<S>(&self, s: &Arc<S>)
    where
        S: Satori + Send + Sync + 'static,
    {
        let s = s.clone();
        let bots = self.bots.clone();

        let addr = format!(
            "ws://{}:{}{}/v1/events",
            self.config.host,
            self.config.port,
            self.config.path.as_deref().unwrap_or_default()
        );
        let (mut ws_stream, _) = connect_async(&addr).await.unwrap();
        info!(target: NET, "WebSocket connected with {addr}");

        let mut seq = 0i64;
        ws_stream
            .send(
                Signal::identify(&self.config.token.clone().unwrap_or_default(), seq)
                    .to_string()
                    .into(),
            )
            .await
            .unwrap();
        let mut interval = tokio::time::interval_at(
            Instant::now() + Duration::from_secs(10),
            Duration::from_secs(10),
        );
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    ws_stream.send(Signal::ping().to_string().into()).await.unwrap();
                }
                data = ws_stream.next() => {
                    trace!(target: NET, "receive ws_msg: {:?}" ,data);
                    match data {
                        Some(Ok(WsMessage::Text(text))) => match serde_json::from_str(&text) {
                            Ok(signal) => match signal {
                                Signal::Event { body: event, .. } => {
                                    info!(target: NET, "receive event: {:?}", event);
                                    // TODO: seq
                                    seq = event.id;
                                    s.handle_event(event);
                                }
                                Signal::Pong { .. } => {}
                                Signal::Ready { body: Logins { logins }, .. } => {
                                    let mut bots = bots.write().await;
                                    for login in logins {
                                        bots.insert(BotId {
                                            platform: login.platform.unwrap(),
                                            id: login.self_id.unwrap(),
                                        });
                                    }
                                }
                                _ => unreachable!(),
                            },
                            Err(e) =>  error!(target: NET, "deserialize error: {e} in {text}"),
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
        if !self.bots.read().await.contains(bot) {
            return Err(SatoriError::InvalidBot);
        }

        let mut req = self
            .client
            .post(format!(
                "http://{}:{}{}/v1/{}",
                self.config.host,
                self.config.port,
                self.config.path.as_deref().unwrap_or_default(),
                payload.method
            ))
            .header("X-Platform", &bot.platform)
            .header("X-Self-ID", &bot.id)
            .json(&payload.body);
        if let Some(token) = &self.config.token {
            req = req.bearer_auth(token);
        }
        trace!(target: NET, ?req);

        let resp = req.send().await.unwrap();
        trace!(target: NET, ?resp);

        match resp.status() {
            StatusCode::OK => Ok(resp.json().await.map_internal_error()?),
            _ => Err(SatoriError::ApiError(ApiError::from_respponse(resp).await?)),
        }
    }

    async fn has_bot(&self, bot: &BotId) -> bool {
        self.bots.read().await.contains(bot)
    }

    async fn get_logins(&self) -> Vec<Login> {
        self.bots
            .read()
            .await
            .iter()
            .map(|bot| Login {
                user: None,
                self_id: Some(bot.id.clone()),
                platform: Some(bot.platform.clone()),
                status: Status::Online,
            })
            .collect()
    }
}
