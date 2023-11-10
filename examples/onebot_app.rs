use satori::{
    impls::onebot11::{Onebot11SDK, Onebot11SDKConfig},
    satori,
};
use tracing_subscriber::filter::LevelFilter;
use url::Host;

mod common;
use common::echo_app::EchoApp;

satori! {
    struct OnebotApp {
        sdk: Onebot11SDK,
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
    let app = OnebotApp::new(
        Onebot11SDK::new(Onebot11SDKConfig {
            host: Host::parse(todo!()).unwrap(),
            ws_port: todo!(),
            http_port: todo!(),
            access_token: None,
            self_id: todo!(),
        }),
        EchoApp {},
    );
    app.start_with_graceful_shutdown(tokio::signal::ctrl_c())
        .await;
}
