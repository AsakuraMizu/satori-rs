use std::{
    net::{IpAddr, SocketAddr},
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

use super::Signal;
use crate::{ApiError, AppT, BotId, CallApiError, Event, Satori, SdkT, SATORI};

type WsMessage = axum::extract::ws::Message;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetAPPConfig {
    pub host: IpAddr,
    pub port: u16,
    pub token: Option<String>,
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
            info!(target: SATORI, "new WebSocket client acceptted.");
            loop {
                tokio::select! {
                    Ok(event) = rx.recv() => {
                        if let Err(e) = socket
                            .send(Signal::event(event).to_string().into())
                            .await
                        {
                            error!(target: SATORI, "Send event error: {e}");
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
                                Ok(Signal::Identify { body, .. }) => socket
                                    .send(Signal::ready(s.s.get_logins().await).to_string().into())
                                    .await
                                    .unwrap(),
                                Ok(_) => unreachable!(),
                                Err(e) => {
                                    error!(target: SATORI, "Receive signal error: {e}")
                                }
                            },
                            _ => {}
                        }
                    }
                    _ = s.stop.cancelled() => {
                        let _ = socket.close().await;
                        break
                    },
                }
            }
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
    ) -> Result<String, CallApiError>
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
        s.call_api(&api, &BotId { platform, id }, &data).await
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
        let app = axum::Router::new()
            .route("/v1/events", axum::routing::get(NetApp::ws_handler))
            .route("/v1/:api", axum::routing::post(NetApp::api_handler))
            .with_state(AppState {
                s: s.clone(),
                tx: self.tx.clone(),
                token: self.config.token.clone(),
            });

        info!(target: SATORI, "Starting server on {}:{}", self.config.host, self.config.port);
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

impl IntoResponse for CallApiError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            Self::ApiError(ApiError::BadRequest(_)) => StatusCode::BAD_REQUEST,
            Self::ApiError(ApiError::Unauthorized) => StatusCode::UNAUTHORIZED,
            Self::ApiError(ApiError::Forbidden) => StatusCode::FORBIDDEN,
            Self::ApiError(ApiError::NotFound) => StatusCode::NOT_FOUND,
            Self::ApiError(ApiError::MethodNotAllowed) => StatusCode::METHOD_NOT_ALLOWED,
            Self::ApiError(ApiError::ServerError(code)) => StatusCode::from_u16(*code).unwrap(),
            Self::InvalidBot => StatusCode::NOT_FOUND,
            Self::JsonError(_) => StatusCode::BAD_REQUEST,
        };
        let body = self.to_string();
        (status, body).into_response()
    }
}
