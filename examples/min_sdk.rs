use std::sync::Arc;

use satori::{
    api::RawApiCall,
    error::{ApiError, SatoriError},
    impls::net::app::{NetAppConfig, NetApp},
    satori,
    structs::{BotId, Login},
    Satori, SatoriSDK,
};
use serde_json::Value;

pub struct Echo {}

impl SatoriSDK for Echo {
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
            s.shutdown().await;
        }
        Err(ApiError::ServerError(500).into())
    }

    async fn has_bot(&self, _bot: &BotId) -> bool {
        true
    }

    async fn get_logins(&self) -> Vec<Login> {
        vec![]
    }
}

satori! {
    struct MinSDK {
        sdk: Echo,
        app: NetApp,
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let sdk = MinSDK::new(
        Echo {},
        NetApp::new(NetAppConfig {
            port: 5141,
            ..Default::default()
        }),
    );
    sdk.start_with_graceful_shutdown(tokio::signal::ctrl_c())
        .await;
}
