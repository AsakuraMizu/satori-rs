use satori::{
    impls::net::sdk::{NetSDK, NetSDKConfig},
    satori,
};
use tracing_subscriber::filter::LevelFilter;

mod common;

use common::echo_app::EchoApp;

satori! {
    struct MinApp {
        sdk: NetSDK,
        app: EchoApp,
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
    let app = MinApp::new(
        NetSDK::new(NetSDKConfig {
            ..Default::default()
        }),
        EchoApp {},
    );
    app.start_with_graceful_shutdown(tokio::signal::ctrl_c())
        .await;
}
