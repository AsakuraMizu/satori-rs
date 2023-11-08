use satori::{
    impls::onebot11::{Onebot11SDK, Onebot11SDKConfig},
    Satori, SATORI,
};
use tracing_subscriber::filter::LevelFilter;
use url::Host;

mod common;

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
    let app = Satori::new(
        Onebot11SDK::new(Onebot11SDKConfig {
            host: Host::parse(todo!()).unwrap(),
            ws_port: todo!(),
            http_port: todo!(),
            access_token: None,
            self_id: todo!(),
        }),
        common::echo_app::EchoApp {},
    );
    app.start().await;
}