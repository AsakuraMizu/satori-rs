use std::sync::Arc;

use satori::{
    api::RawApiCall,
    error::{ApiError, SatoriError},
    impls::net::app::{NetAPPConfig, NetApp},
    structs::{BotId, Login},
    AppT, Satori, SdkT,
};
use serde_json::Value;
use tracing_subscriber::filter::LevelFilter;

pub struct Echo {}

impl SdkT for Echo {
    async fn start<S, A>(&self, _s: &Arc<Satori<S, A>>)
    where
        S: SdkT + Send + Sync + 'static,
        A: AppT + Send + Sync + 'static,
    {
    }

    async fn call_api<S, A>(
        &self,
        s: &Arc<Satori<S, A>>,
        _bot: &BotId,
        payload: RawApiCall,
    ) -> Result<Value, SatoriError>
    where
        S: SdkT + Send + Sync + 'static,
        A: AppT + Send + Sync + 'static,
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

#[tokio::main]
async fn main() {
    let filter = tracing_subscriber::filter::Targets::new().with_default(LevelFilter::INFO);
    use tracing_subscriber::{
        prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, Layer,
    };
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(filter))
        .init();
    let sdk = Satori::new(
        Echo {},
        NetApp::new(NetAPPConfig {
            port: 5141,
            ..Default::default()
        }),
    );
    sdk.start().await;
}
