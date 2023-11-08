use std::sync::Arc;

use satori::{
    structs::{BotId, ChannelType, Event},
    AppT, Satori, SdkT,
};
use tracing::debug;

pub struct EchoApp {}

impl AppT for EchoApp {
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
