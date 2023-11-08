use satori::{
    impls::net::sdk::{NetSDK, NetSDKConfig},
    Satori,
};
use tracing_subscriber::filter::LevelFilter;

mod common;

#[tokio::main]
async fn main() {
    let filter = tracing_subscriber::filter::Targets::new().with_default(LevelFilter::INFO);
    use tracing_subscriber::{
        prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, Layer,
    };
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(filter))
        .init();
    let app = Satori::new(
        NetSDK::new(NetSDKConfig {
            ..Default::default()
        }),
        common::echo_app::EchoApp {},
    );
    app.start().await;
}
