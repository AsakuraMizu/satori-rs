use std::sync::Arc;

use satori::{
    api::SatoriApi,
    structs::{BotId, ChannelType, Event},
    Satori, SatoriApp,
};
use tracing::debug;

pub struct EchoApp {}

impl SatoriApp for EchoApp {
    async fn start<S>(&self, _s: &Arc<S>)
    where
        S: Satori + Send + Sync + 'static,
    {
    }

    async fn handle_event<S>(&self, s: &Arc<S>, event: Event)
    where
        S: Satori + Send + Sync + 'static,
    {
        if let Some(user) = &event.user {
            if user.id == event.self_id {
                debug!("self event");
                return;
            }
        }
        if let Some(message) = event.message {
            if let Some(content) = &message.content {
                if content.starts_with("echo") {
                    let bot = BotId {
                        id: event.self_id,
                        platform: event.platform,
                    };
                    if let Some(ch) = event.channel {
                        match ch.ty {
                            Some(ChannelType::Text) => {
                                let r = s.create_message(&bot, ch.id, content.clone()).await;
                                debug!("api response:{:?}", r);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}
