use std::{future::Future, sync::Arc};

use serde_json::Value;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};

use crate::{
    api::{IntoRawApiCall, RawApiCall},
    error::SatoriError,
    structs::{BotId, Event, Login},
};

pub trait SdkT {
    fn start<S, A>(&self, s: &Arc<Satori<S, A>>) -> impl Future<Output = ()> + Send
    where
        S: SdkT + Send + Sync + 'static,
        A: AppT + Send + Sync + 'static;

    fn call_api<S, A>(
        &self,
        s: &Arc<Satori<S, A>>,
        bot: &BotId,
        payload: RawApiCall,
    ) -> impl Future<Output = Result<Value, SatoriError>> + Send
    where
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

const SATORI: &str = "Satori";

pub struct Satori<S, A> {
    s: S,
    a: A,
    pub stop: CancellationToken,
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
        info!(target: SATORI, "Starting...");
        let _ = tokio::join!(self.s.start(self), self.a.start(self));
    }

    pub fn shutdown(self: &Arc<Self>) {
        info!(target: SATORI, "Stopping...");
        self.stop.cancel();
    }

    pub async fn call_api<T>(
        self: &Arc<Self>,
        bot: &BotId,
        payload: T,
    ) -> Result<Value, SatoriError>
    where
        T: IntoRawApiCall + Send,
    {
        let payload = payload.into_raw();
        debug!(target: SATORI, ?bot, ?payload, "call api");
        self.s.call_api(self, bot, payload).await
    }

    pub async fn handle_event(self: &Arc<Self>, event: Event) {
        debug!(target: SATORI, ?event, "handle event");
        self.a.handle_event(self, event).await
    }

    pub async fn get_logins(self: &Arc<Self>) -> Vec<Login> {
        self.s.get_logins().await
    }
}
