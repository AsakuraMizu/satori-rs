use std::sync::Arc;

use satori::{
    net::app::{NetAPPConfig, NetApp},
    ApiError, AppT, BotId, CallApiError, Login, Satori, SdkT, SATORI,
};
use serde::{Serialize, de::DeserializeOwned};
use tracing::info;
use tracing_subscriber::filter::LevelFilter;

pub struct Echo {}

impl SdkT for Echo {
    async fn start<S, A>(&self, _s: &Arc<Satori<S, A>>)
    where
        S: SdkT + Send + Sync + 'static,
        A: AppT + Send + Sync + 'static,
    {
    }

    async fn call_api<T, R, S, A>(
        &self,
        s: &Arc<Satori<S, A>>,
        api: &str,
        bot: &BotId,
        data: T,
    ) -> Result<R, CallApiError>
    where
        T: Serialize + Send,
        R: DeserializeOwned,
        S: SdkT + Send + Sync + 'static,
        A: AppT + Send + Sync + 'static,
    {
        let data_str = serde_json::to_string(&data).unwrap();
        info!(?bot, "{api}({data_str})");
        if api == "stop" {
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
    let filter = tracing_subscriber::filter::Targets::new()
        .with_default(LevelFilter::INFO)
        .with_targets([(SATORI, LevelFilter::TRACE)]);
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
