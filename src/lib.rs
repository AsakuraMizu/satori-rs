#![feature(concat_idents)]

mod structs;
pub use structs::*;

use std::future::Future;
use std::sync::Arc;

use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;
use tokio_util::sync::CancellationToken;

pub const SATORI: &str = "Satori";

pub struct Satori<S, A> {
    s: S,
    a: A,
    stop: CancellationToken,
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("bad request")]
    BadRequest(#[from] anyhow::Error),
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("method not allowed")]
    MethodNotAllowed,
    #[error("server error ({0})")]
    ServerError(u16),
}

#[derive(Debug, Error)]
pub enum CallApiError {
    #[error(transparent)]
    ApiError(#[from] ApiError),
    #[error("invalid bot")]
    InvalidBot,
    #[error("unknown error: {0}")]
    UnknownError(#[from] anyhow::Error),
}

pub trait SdkT {
    fn start<S, A>(&self, s: &Arc<Satori<S, A>>) -> impl Future<Output = ()> + Send
    where
        S: SdkT + Send + Sync + 'static,
        A: AppT + Send + Sync + 'static;

    fn call_api<T, R, S, A>(
        &self,
        s: &Arc<Satori<S, A>>,
        api: &str,
        bot: &BotId,
        data: T,
    ) -> impl Future<Output = Result<R, CallApiError>> + Send
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned,
        S: SdkT + Send + Sync + 'static,
        A: AppT + Send + Sync + 'static;

    fn has_bot(&self, bot: &BotId) -> impl Future<Output = bool> + Send;

    fn get_logins(&self) -> impl Future<Output = Vec<Login>> + Send;
}

pub trait AppT {
    fn start<S, A>(&self, s: &Arc<Satori<S, A>>) -> impl Future<Output = ()> + Send
    where
        S: SdkT + Send + Sync + 'static,
        A: AppT + Send + Sync + 'static;

    fn handle_event<S, A>(
        &self,
        s: &Arc<Satori<S, A>>,
        event: Event,
    ) -> impl Future<Output = ()> + std::marker::Send
    where
        S: SdkT + Send + Sync + 'static,
        A: AppT + Send + Sync + 'static;
}

impl<S, A> Satori<S, A>
where
    S: SdkT + Send + Sync + 'static,
    A: AppT + Send + Sync + 'static,
{
    pub fn new(s: S, a: A) -> Arc<Self> {
        Arc::new(Self {
            s,
            a,
            stop: CancellationToken::new(),
        })
    }

    pub async fn start(self: &Arc<Self>) {
        let _ = tokio::join!(self.s.start(self), self.a.start(self));
    }

    pub fn shutdown(self: &Arc<Self>) {
        self.stop.cancel();
    }

    pub async fn call_api<T, R>(
        self: &Arc<Self>,
        api: &str,
        bot: &BotId,
        data: T,
    ) -> Result<R, CallApiError>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        self.s.call_api(self, api, bot, data).await
        // self.s.call_api(self, api, bot, data).await.and_then(|s| {
        //     tracing::trace!(target:SATORI, "recive api resp:{s}");
        //     Ok(serde_json::from_str(&s)?)
        // })
    }

    pub async fn handle_event(self: &Arc<Self>, event: Event) {
        self.a.handle_event(self, event).await
    }
}

mod impls;
pub use impls::*;


#[cfg(feature = "message")]
pub mod message;
