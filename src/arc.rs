use std::sync::Arc;

use serde::Serialize;

use crate::{AppT, BotId, CallApiError, Event, Login, Satori, SdkT};

impl<Inner> AppT for Arc<Inner>
where
    Inner: AppT + Send + Sync + 'static,
{
    async fn start<S, A>(&self, s: &Arc<Satori<S, A>>)
    where
        S: SdkT + Send + Sync + 'static,
        A: AppT + Send + Sync + 'static,
    {
        let me = self.clone();
        let s = s.clone();
        tokio::spawn(async move { me.as_ref().start(&s).await })
            .await
            .unwrap();
    }

    async fn handle_event<S, A>(&self, s: &Arc<Satori<S, A>>, event: Event)
    where
        S: SdkT + Send + Sync + 'static,
        A: AppT + Send + Sync + 'static,
    {
        let me = self.clone();
        let s = s.clone();
        tokio::spawn(async move { me.as_ref().handle_event(&s, event).await })
            .await
            .unwrap()
    }
}

impl<Inner> SdkT for Arc<Inner>
where
    Inner: SdkT + Send + Sync + 'static,
{
    async fn start<S, A>(&self, s: &Arc<Satori<S, A>>)
    where
        S: SdkT + Send + Sync + 'static,
        A: AppT + Send + Sync + 'static,
    {
        let me = self.clone();
        let s = s.clone();
        tokio::spawn(async move { me.as_ref().start(&s).await })
            .await
            .unwrap();
    }

    async fn call_api<T, S, A>(
        &self,
        s: &Arc<Satori<S, A>>,
        api: &str,
        bot: &BotId,
        data: T,
    ) -> Result<String, CallApiError>
    where
        T: Serialize + Send + Sync,
        S: SdkT + Send + Sync + 'static,
        A: AppT + Send + Sync + 'static,
    {
        self.as_ref().call_api(s, api, bot, data).await
    }

    async fn has_bot(&self, bot: &BotId) -> bool {
        self.as_ref().has_bot(bot).await
    }

    async fn get_logins(&self) -> Vec<Login> {
        self.as_ref().get_logins().await
    }
}
