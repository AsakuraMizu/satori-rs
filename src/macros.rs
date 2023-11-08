use std::{future::Future, sync::Arc};

use serde_json::Value;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};

use crate::{
    api::IntoRawApiCall,
    error::SatoriError,
    structs::{BotId, Event, Login},
    Satori, SatoriApp, SatoriSdk, SATORI,
};

pub struct SatoriImpl<S, A> {
    s: S,
    a: A,
    stop: CancellationToken,
}

impl<S, A> SatoriImpl<S, A> {
    pub fn new(s: S, a: A) -> Arc<Self> {
        Arc::new(Self {
            s,
            a,
            stop: CancellationToken::new(),
        })
    }
}

impl<S, A> Satori for SatoriImpl<S, A>
where
    S: SatoriSdk + Send + Sync + 'static,
    A: SatoriApp + Send + Sync + 'static,
{
    async fn start(self: &Arc<Self>) {
        info!(target: SATORI, "Starting...");
        let _ = tokio::join!(
            tokio::spawn({
                let me = self.clone();
                async move { me.a.start(&me).await }
            }),
            tokio::spawn({
                let me = self.clone();
                async move { me.s.start(&me).await }
            }),
        );
    }

    fn shutdown(self: &Arc<Self>) {
        info!(target: SATORI, "Stopping...");
        self.stop.cancel();
    }

    async fn call_api<T>(self: &Arc<Self>, bot: &BotId, payload: T) -> Result<Value, SatoriError>
    where
        T: IntoRawApiCall + Send,
    {
        let payload = payload.into_raw();
        debug!(target: SATORI, ?bot, ?payload, "call api");
        self.s.call_api(self, bot, payload).await
    }

    async fn handle_event(self: &Arc<Self>, event: Event) {
        debug!(target: SATORI, ?event, "handle event");
        self.a.handle_event(self, event).await
    }

    async fn get_logins(self: &Arc<Self>) -> Vec<Login> {
        self.s.get_logins().await
    }

    async fn stopped(self: &Arc<Self>) {
        self.stop.cancelled().await
    }
}
#[cfg(feature = "graceful-shutdown")]
impl<S, A> SatoriImpl<S, A>
where
    S: SatoriSdk + Send + Sync + 'static,
    A: SatoriApp + Send + Sync + 'static,
{
    pub async fn start_with_graceful_shutdown(self: &Arc<Self>, signal: impl Future) {
        tokio::select! {
            _ = self.start() => {}
            _ = signal => {
                self.shutdown();
            }
        }
    }
}

macro_rules! satori {
    () => {};
}
