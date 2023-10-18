use std::{net::IpAddr, str::FromStr, sync::Arc};

use satori::{
    net::app::{NetAPPConfig, NetApp},
    ApiError, AppT, BotId, CallApiError, Login, Satori, SdkT,
};
use serde::Serialize;
use tracing_subscriber::filter::LevelFilter;

pub struct Echo {}

impl SdkT for Echo {
    async fn start<S, A>(&self, _s: &Arc<Satori<S, A>>)
    where
        S: SdkT + Send + Sync + 'static,
        A: AppT + Send + Sync + 'static,
    {
    }

    async fn call_api<T, S, A>(
        &self,
        s: &Arc<Satori<S, A>>,
        api: &str,
        _bot: &BotId,
        _data: T,
    ) -> Result<String, CallApiError>
    where
        T: Serialize + Send,
        S: SdkT + Send + Sync + 'static,
        A: AppT + Send + Sync + 'static,
    {
        if api == "stop" {
            s.shutdown();
        }
        Err(ApiError::ServerError(500).into())
    }

    async fn get_logins(&self) -> Vec<Login> {
        vec![]
    }
}

#[tokio::main]
async fn main() {
    let filter = tracing_subscriber::filter::Targets::new()
        .with_default(LevelFilter::INFO)
        .with_targets([("Satori", LevelFilter::TRACE)]);
    use tracing_subscriber::{
        prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, Layer,
    };
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(filter))
        .init();
    let sdk = Satori::new(
        Echo {},
        NetApp::new(NetAPPConfig {
            host: IpAddr::from_str("127.0.0.1").unwrap(),
            port: 5141,
            token: None,
        }),
    );
    sdk.start().await;
}
