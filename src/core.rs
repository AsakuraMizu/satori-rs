use std::{future::Future, sync::Arc};

use serde_json::Value;

use crate::{
    api::{IntoRawApiCall, RawApiCall},
    error::SatoriError,
    structs::{BotId, Event, Login},
};

pub trait SatoriSDK {
    fn start<S>(&self, s: &Arc<S>) -> impl Future<Output = ()> + Send
    where
        S: Satori + Send + Sync + 'static;

    fn call_api<S>(
        &self,
        s: &Arc<S>,
        bot: &BotId,
        payload: RawApiCall,
    ) -> impl Future<Output = Result<Value, SatoriError>> + Send
    where
        S: Satori + Send + Sync + 'static;

    fn has_bot(&self, bot: &BotId) -> impl Future<Output = bool> + Send;

    fn get_logins(&self) -> impl Future<Output = Vec<Login>> + Send;
}

pub trait SatoriApp {
    fn start<S>(&self, s: &Arc<S>) -> impl Future<Output = ()> + Send
    where
        S: Satori + Send + Sync + 'static;

    fn handle_event<S>(&self, s: &Arc<S>, event: Event) -> impl Future<Output = ()> + Send
    where
        S: Satori + Send + Sync + 'static;
}

pub trait Satori {
    fn spawn(self: &Arc<Self>) -> impl Future<Output = ()> + Send;
    fn start(self: &Arc<Self>) -> impl Future<Output = ()> + Send;
    fn shutdown(self: &Arc<Self>) -> impl Future<Output = ()> + Send;

    fn call_api<T>(
        self: &Arc<Self>,
        bot: &BotId,
        payload: T,
    ) -> impl Future<Output = Result<Value, SatoriError>> + Send
    where
        T: IntoRawApiCall + Send;
    fn handle_event(self: &Arc<Self>, event: Event) -> impl Future<Output = ()> + Send;
    fn get_logins(self: &Arc<Self>) -> impl Future<Output = Vec<Login>> + Send;

    fn stopped(self: &Arc<Self>) -> impl Future<Output = ()> + Send;
}
