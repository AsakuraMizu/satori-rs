use std::sync::Arc;

use satori::{
    api::RawApiCall,
    error::{ApiError, SatoriError},
    impls::net::app::{NetAPPConfig, NetApp},
    satori,
    structs::{BotId, Login},
    Satori, SatoriSdk,
};
use serde_json::Value;
use tracing_subscriber::filter::LevelFilter;

pub struct Echo {}

impl SatoriSdk for Echo {
    async fn start<S>(&self, _s: &Arc<S>)
    where
        S: Satori + Send + Sync + 'static,
    {
    }

    async fn call_api<S>(
        &self,
        s: &Arc<S>,
        _bot: &BotId,
        payload: RawApiCall,
    ) -> Result<Value, SatoriError>
    where
        S: Satori + Send + Sync + 'static,
    {
        if payload.method == "stop" {
            s.shutdown();
        }
        Err(ApiError::ServerError(500).into())
    }

    async fn has_bot(&self, _bot: &BotId) -> bool {
        false
    }

    async fn get_logins(&self) -> Vec<Login> {
        vec![]
    }
}

satori! {
    struct MinSdk {
        sdk: Echo,
        app: NetApp,
    }
}

#[tokio::main]
async fn main() {
    let filter = tracing_subscriber::filter::Targets::new().with_default(LevelFilter::INFO);
    use tracing_subscriber::{
        prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, Layer,
    };
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(filter))
        .init();
    let sdk = MinSdk::new(
        Echo {},
        NetApp::new(NetAPPConfig {
            port: 5141,
            ..Default::default()
        }),
    );
    sdk.start_with_graceful_shutdown(tokio::signal::ctrl_c())
        .await;
}
