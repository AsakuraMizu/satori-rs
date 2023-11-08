use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{
    extract::{FromRef, Path, State, WebSocketUpgrade},
    response::IntoResponse,
    Json, TypedHeader,
};
use futures_util::StreamExt;
use headers::{authorization::Bearer, Authorization, Header};
use http::{HeaderName, HeaderValue, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::broadcast;
use tracing::{error, info};

use super::{Signal, NET};
use crate::{
    api::RawApiCall,
    error::{ApiError, SatoriError},
    structs::{BotId, Event},
    AppT, Satori, SdkT,
};

type WsMessage = axum::extract::ws::Message;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetAPPConfig {
    pub host: IpAddr,
    pub port: u16,
    pub path: Option<String>,
    pub token: Option<String>,
}

impl Default for NetAPPConfig {
    fn default() -> Self {
        Self {
            host: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 5140,
            path: None,
            token: None,
        }
    }
}

#[derive(Debug)]
pub struct NetApp {
    config: NetAPPConfig,
    tx: broadcast::Sender<Event>,
}

impl NetApp {
    pub fn new(config: NetAPPConfig) -> Self {
        let (tx, _) = broadcast::channel(128);
        Self { config, tx }
    }

    async fn ws_handler<S, A>(
        ws: WebSocketUpgrade,
        State(s): State<Arc<Satori<S, A>>>,
        State(token): State<Option<String>>,
        State(tx): State<broadcast::Sender<Event>>,
    ) -> impl IntoResponse
    where
        S: SdkT + Send + Sync + 'static,
        A: AppT + Send + Sync + 'static,
    {
        let mut rx = tx.subscribe();
        ws.on_upgrade(|mut socket| async move {
            info!(target: NET, "new WebSocket client acceptted.");
            loop {
                tokio::select! {
                    Ok(event) = rx.recv() => {
                        if let Err(e) = socket
                            .send(Signal::event(event).to_string().into())
                            .await
                        {
                            error!(target: NET, "Send event error: {:?}", e);
                            break;
                        }
                    }
                    Some(Ok(msg)) = socket.next() => {
                        match msg {
                            WsMessage::Close(_) => break,
                            WsMessage::Ping(b) => {
                                socket.send(WsMessage::Pong(b)).await.ok();
                            }
                            WsMessage::Text(text) => match serde_json::from_str(&text) {
                                Ok(Signal::Ping { .. }) => socket
                                    .send(Signal::pong().to_string().into())
                                    .await
                                    .unwrap(), //todo
                                Ok(Signal::Identify { body, .. }) => {
                                    if let Some(token) = &token {
                                        if body.token.as_ref() != Some(token) {
                                            break
                                        }
                                    }
                                    socket
                                        .send(Signal::ready(s.get_logins().await).to_string().into())
                                        .await
                                        .unwrap();
                                },
                                Ok(_) => unreachable!(),
                                Err(e) => {
                                    error!(target: NET, "Receive signal error: {:?}", e)
                                }
                            },
                            _ => {}
                        }
                    }
                    _ = s.stop.cancelled() => break
                }
            }
            let _ = socket.close().await;
        })
    }

    async fn api_handler<S, A>(
        Path(api): Path<String>,
        TypedHeader(Platform(platform)): TypedHeader<Platform>,
        TypedHeader(SelfID(id)): TypedHeader<SelfID>,
        bearer: Option<TypedHeader<Authorization<Bearer>>>,
        State(s): State<Arc<Satori<S, A>>>,
        State(token): State<Option<String>>,
        Json(data): Json<Value>,
    ) -> Result<String, SatoriError>
    where
        S: SdkT + Send + Sync + 'static,
        A: AppT + Send + Sync + 'static,
    {
        if let Some(token) = token {
            let Some(TypedHeader(Authorization(bearer))) = bearer else {
                return Err(ApiError::Unauthorized.into());
            };
            if bearer.token() != token {
                return Err(ApiError::Forbidden.into());
            }
        }
        s.call_api(
            &BotId { platform, id },
            RawApiCall {
                method: api,
                body: data,
            },
        )
        .await
        .map(|v| v.to_string())
    }
}

struct AppState<S, A> {
    s: Arc<Satori<S, A>>,
    tx: broadcast::Sender<Event>,
    token: Option<String>,
}

impl<S, A> Clone for AppState<S, A> {
    fn clone(&self) -> Self {
        Self {
            s: self.s.clone(),
            tx: self.tx.clone(),
            token: self.token.clone(),
        }
    }
}

impl<S, A> FromRef<AppState<S, A>> for Arc<Satori<S, A>> {
    fn from_ref(input: &AppState<S, A>) -> Self {
        input.s.clone()
    }
}

impl<S, A> FromRef<AppState<S, A>> for broadcast::Sender<Event> {
    fn from_ref(input: &AppState<S, A>) -> Self {
        input.tx.clone()
    }
}

impl<S, A> FromRef<AppState<S, A>> for Option<String> {
    fn from_ref(input: &AppState<S, A>) -> Self {
        input.token.clone()
    }
}

impl AppT for NetApp {
    async fn start<S, A>(&self, s: &Arc<Satori<S, A>>)
    where
        S: SdkT + Send + Sync + 'static,
        A: AppT + Send + Sync + 'static,
    {
        let path = self.config.path.as_deref().unwrap_or("");
        let app = axum::Router::new()
            .route(
                &format!("{}/v1/events", &path),
                axum::routing::get(NetApp::ws_handler),
            )
            .route(
                &format!("{}/v1/:api", &path),
                axum::routing::post(NetApp::api_handler),
            )
            .with_state(AppState {
                s: s.clone(),
                tx: self.tx.clone(),
                token: self.config.token.clone(),
            });

        info!(target: NET, "Starting server on {}:{}", self.config.host, self.config.port);
        let _ = axum::Server::bind(&(self.config.host, self.config.port).into())
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .with_graceful_shutdown(s.stop.cancelled())
            .await;
    }

    async fn handle_event<S, A>(&self, _s: &Arc<Satori<S, A>>, event: Event)
    where
        S: SdkT + Send + Sync + 'static,
        A: AppT + Send + Sync + 'static,
    {
        self.tx.send(event).ok();
    }
}

static HEADER_PLATFORM: HeaderName = HeaderName::from_static("x-platform");
struct Platform(String);
impl Header for Platform {
    fn name() -> &'static HeaderName {
        &HEADER_PLATFORM
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values.next().ok_or_else(headers::Error::invalid)?;

        Ok(Self(
            value
                .to_str()
                .map_err(|_| headers::Error::invalid())?
                .to_string(),
        ))
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(Some(HeaderValue::from_str(&self.0).unwrap()))
    }
}
static HEADER_SELF_ID: HeaderName = HeaderName::from_static("x-self-id");
struct SelfID(String);
impl Header for SelfID {
    fn name() -> &'static HeaderName {
        &HEADER_SELF_ID
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values.next().ok_or_else(headers::Error::invalid)?;

        Ok(Self(
            value
                .to_str()
                .map_err(|_| headers::Error::invalid())?
                .to_string(),
        ))
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(Some(HeaderValue::from_str(&self.0).unwrap()))
    }
}

impl IntoResponse for SatoriError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            Self::ApiError(ApiError::BadRequest(_)) => StatusCode::BAD_REQUEST,
            Self::ApiError(ApiError::Unauthorized) => StatusCode::UNAUTHORIZED,
            Self::ApiError(ApiError::Forbidden) => StatusCode::FORBIDDEN,
            Self::ApiError(ApiError::NotFound) => StatusCode::NOT_FOUND,
            Self::ApiError(ApiError::MethodNotAllowed) => StatusCode::METHOD_NOT_ALLOWED,
            Self::ApiError(ApiError::ServerError(code)) => StatusCode::from_u16(*code).unwrap(),
            Self::InvalidBot => StatusCode::NOT_FOUND,
            Self::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = self.to_string();
        (status, body).into_response()
    }
}
