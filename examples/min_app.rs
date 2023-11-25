use satori::{
    impls::net::sdk::{NetSDK, NetSDKConfig},
    satori,
};

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
    tracing_subscriber::fmt::init();

    let app = MinApp::new(
        NetSDK::new(NetSDKConfig {
            ..Default::default()
        }),
        EchoApp {},
    );
    app.start_with_graceful_shutdown(tokio::signal::ctrl_c())
        .await;
}
