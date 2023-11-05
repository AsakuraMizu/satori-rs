use std::{net::IpAddr, sync::Arc};

use futures_util::{SinkExt, StreamExt};
use http::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::client::IntoClientRequest};
use tracing::{info, trace};

use crate::{
    api::IntoRawApiCall,
    error::SatoriError,
    structs::{BotId, Login},
    AppT, Satori, SdkT,
};

type WsMessage = tokio_tungstenite::tungstenite::Message;

const ONEBOT: &str = "OneBot";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Onebot11SDKConfig {
    pub host: IpAddr,
    pub port: u16,
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
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    fn url(&self, api: &str) -> String {
        format!("http://{}:{}/{}", self.config.host, self.config.port, api)
    }
}

impl SdkT for Onebot11SDK {
    async fn start<S, A>(&self, s: &std::sync::Arc<Satori<S, A>>) -> ()
    where
        S: SdkT + Send + Sync + 'static,
        A: AppT + Send + Sync + 'static,
    {
        let addr = format!("ws://{}:{}/", self.config.host, self.config.port);
        let mut req = addr.as_str().into_client_request().unwrap();
        if let Some(access_token) = &self.config.access_token {
            req.headers_mut().append(
                AUTHORIZATION,
                format!("Bearer {}", access_token).try_into().unwrap(),
            );
        }
        let (mut ws_stream, _) = connect_async(req).await.unwrap();
        info!(target:ONEBOT, "WebSocket connected with {addr}");

        loop {
            tokio::select! {
               data = ws_stream.next() => {
                   trace!(target: ONEBOT, "receive ws_msg: {:?}" ,data);
                   match data {
                       Some(Ok(WsMessage::Text(text))) => {
                           info!(target: ONEBOT, "{text}");
                       }
                       Some(Ok(WsMessage::Ping(d))) => match ws_stream.send(WsMessage::Pong(d)).await {
                           Ok(_) => {}
                           Err(_) => break,
                       }
                       Some(Ok(WsMessage::Pong(_))) => {}
                       _ => break,
                   }
               }
               _ = s.stop.cancelled() => {
                   ws_stream.send(WsMessage::Close(None)).await.ok();
                   break;
               }
            }
        }
    }

    async fn call_api<T, S, A>(
        &self,
        s: &Arc<Satori<S, A>>,
        bot: &BotId,
        payload: T,
    ) -> Result<String, SatoriError>
    where
        T: IntoRawApiCall + Send,
        S: SdkT + Send + Sync + 'static,
        A: AppT + Send + Sync + 'static,
    {
        todo!()
    }

    async fn has_bot(&self, bot: &BotId) -> bool {
        bot.platform == ONEBOT && bot.id == self.config.self_id
    }

    async fn get_logins(&self) -> Vec<Login> {
        vec![]
    }
}
