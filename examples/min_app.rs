use std::{
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
    sync::Arc,
};

use satori::{
    api::TypedApiCall,
    impls::net::sdk::{NetSDK, NetSDKConfig},
    structs::{BotId, ChannelType, Event},
    AppT, Satori, SdkT, SATORI,
};
use serde_json::{json, Value};
use tracing::{debug, info};
use tracing_subscriber::filter::LevelFilter;

pub struct Echo {}

impl AppT for Echo {
    async fn start<S, A>(&self, _s: &Arc<Satori<S, A>>)
    where
        S: SdkT + Send + Sync + 'static,
        A: AppT + Send + Sync + 'static,
    {
    }

    async fn handle_event<S, A>(&self, s: &Arc<Satori<S, A>>, event: Event)
    where
        S: SdkT + Send + Sync + 'static,
        A: AppT + Send + Sync + 'static,
    {
        info!("start_handle_evnet");
        if let Some(user) = event.user {
            if user.id == event.self_id {
                info!("self event");
                return;
            }
        }
        if let Some(message) = event.message {
            if let Some(content) = &message.content {
                info!(
                    "try to parse message: {:?}",
                    satori::message::from_str(&content)
                );
                if content.starts_with("echo") {
                    let bot = BotId {
                        id: event.self_id,
                        platform: event.platform,
                    };
                    if let Some(ch) = event.channel {
                        match ch.ty {
                            Some(ChannelType::Text) => {
                                let r = s.create_message(&bot, &ch.id, &content).await;
                                debug!("api response:{:?}", r);
                            }
                            // ChannelType::Direct => {
                            //     let _ch = s
                            //         .call_api::<Channel>(
                            //             "user.channel.create",
                            //             &bot,
                            //             json!({
                            //                 "user_id": ch.id,
                            //             }),
                            //         )
                            //         .await
                            //         .unwrap();
                            // }
                            _ => {}
                        }
                    }
                }
            }
        }
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
    let app = Satori::new(
        NetSDK::new(NetSDKConfig {
            ..Default::default()
        }),
        Echo {},
    );
    app.start().await;
}
