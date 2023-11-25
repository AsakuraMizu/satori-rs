use satori::{
    impls::onebot11::{Onebot11SDK, Onebot11SDKConfig},
    satori,
};

mod common;
use common::echo_app::EchoApp;

satori! {
    struct OnebotApp {
        sdk: Onebot11SDK,
        app: (EchoApp, EchoApp),
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = OnebotApp::new(
        Onebot11SDK::new(Onebot11SDKConfig {
            host: todo!(),
            port: todo!(),
            access_token: None,
            self_id: todo!(),
        }),
        (EchoApp {}, EchoApp {}),
    );
    app.start_with_graceful_shutdown(tokio::signal::ctrl_c())
        .await;
}
