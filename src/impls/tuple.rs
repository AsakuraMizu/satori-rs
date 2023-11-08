use std::sync::Arc;

use serde_json::Value;

use crate::{
    api::IntoRawApiCall,
    error::SatoriError,
    structs::{BotId, Event, Login},
    AppT, Satori, SdkT,
};

macro_rules! impl_appt_for_tuples {
    ($($i:tt $t:tt),+) => {
        impl<$($t),*> AppT for ($($t,)*)
        where
            $($t: AppT + Send + Sync + 'static,)*
        {
            async fn start<S, A>(&self, s: &Arc<Satori<S, A>>)
            where
                S: SdkT + Send + Sync + 'static,
                A: AppT + Send + Sync + 'static,
            {
                tokio::join!($(self.$i.start(s)),*);
            }

            async fn handle_event<S, A>(&self, s: &Arc<Satori<S, A>>, event: Event)
            where
                S: SdkT + Send + Sync + 'static,
                A: AppT + Send + Sync + 'static,
            {
                tokio::join!(
                    $(self.$i.handle_event(s, event.clone())),+
                );
            }
        }
    };
}

impl_appt_for_tuples!(0 A0);
impl_appt_for_tuples!(0 A0, 1 A1);
impl_appt_for_tuples!(0 A0, 1 A1, 2 A2);
impl_appt_for_tuples!(0 A0, 1 A1, 2 A2, 3 A3);
impl_appt_for_tuples!(0 A0, 1 A1, 2 A2, 3 A3, 4 A4);
impl_appt_for_tuples!(0 A0, 1 A1, 2 A2, 3 A3, 4 A4, 5 A5);

macro_rules! impl_sdkt_for_tuples {
    ($($i:tt $t:tt $r: tt),+) => {
        impl<$($t),*> SdkT for ($($t,)*)
        where
            $($t: SdkT + Send + Sync + 'static,)*
        {
            async fn start<S, A>(&self, s: &Arc<Satori<S, A>>)
            where
                S: SdkT + Send + Sync + 'static,
                A: AppT + Send + Sync + 'static,
            {
                tokio::join!($(self.$i.start(s)),*);
            }

            async fn call_api<T, S, A>(
                &self,
                s: &Arc<Satori<S, A>>,
                bot: &BotId,
                payload: T,
            ) -> Result<Value, SatoriError>
            where
                T: IntoRawApiCall + Send,
                S: SdkT + Send + Sync + 'static,
                A: AppT + Send + Sync + 'static,
            {
                tokio::select! {
                    $(true = self.$i.has_bot(bot) => self.$i.call_api(s, bot, payload).await,)*
                    else => Err(SatoriError::InvalidBot),
                }
            }

            async fn has_bot(&self, bot: &BotId) -> bool {
                tokio::select! {
                    $(true = self.$i.has_bot(bot) => true,)*
                    else => false,
                }
            }

            async fn get_logins(&self) -> Vec<Login> {
                let ($($r,)*) = tokio::join!($(self.$i.get_logins()),*);
                [$($r),*].concat()
            }
        }
    };
}

impl_sdkt_for_tuples!(0 S0 r0);
impl_sdkt_for_tuples!(0 S0 r0, 1 S1 r1);
impl_sdkt_for_tuples!(0 S0 r0, 1 S1 r1, 2 S2 r2);
impl_sdkt_for_tuples!(0 S0 r0, 1 S1 r1, 2 S2 r2, 3 S3 r3);
impl_sdkt_for_tuples!(0 S0 r0, 1 S1 r1, 2 S2 r2, 3 S3 r3, 4 S4 r4);
impl_sdkt_for_tuples!(0 S0 r0, 1 S1 r1, 2 S2 r2, 3 S3 r3, 4 S4 r4, 5 S5 r5);
