// #[macro_export]
// #[doc(hidden)]
// macro_rules! __satori_assert_impl {
//     (( $($type:ty),+ $(,)? ): $tr:path) => {
//         $($crate::__satori_assert_impl!($type: $tr);)+
//     };
//     ($type:ty: $tr:path) => {
//         const _: () = {
//             const fn assert_impl<T: $tr + std::marker::Send + std::marker::Sync + 'static>() {}
//             assert_impl::<$type>();
//         };
//     };
// }

#[macro_export]
#[doc(hidden)]
macro_rules! __satori_wrap_tuple {
    (( $($t:ty),+ $(,)? )) => {
        ( $($t,)+ )
    };
    ($t:ty) => {
        $crate::__satori_wrap_tuple!(($t))
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __satori_convert_tuple {
    (( $($t:ty),+ $(,)? ), $input:expr) => {
        $input
    };
    ($t:ty, $input:expr) => {
        ($input,)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __satori_expand {
    ($cb:ident, ( $($arg:tt),* ), @ { ( $($count:tt)* ) $($t:tt)* }) => {
        $crate::$cb!(( $($arg),* ), $($t)*)
    };
    ($cb:ident, ( $($arg:tt),* ), @ { ( $($count:tt)* ) $($t:tt)* } $e:tt, $($r:tt)*) => {
        $crate::__satori_expand!($cb, ( $($arg),* ), @ { ( $($count)* _ ) $($t)* ($($count)*) $e, } $($r)*)
    };
    ($cb:ident, ( $($arg:tt),* ), ($($e:tt),* $(,)?)) => {
        $crate::__satori_expand!($cb, ( $($arg),* ), @ { () } $($e,)*)
    };
    ($cb:ident, ( $($arg:tt),* ), $e:tt) => {
        $crate::__satori_expand!($cb, ( $($arg),* ), @ { () } $e,)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __satori_impl_start_sdk {
    ( ( $self:ident, $set:ident ), $( ( $($skip:tt)* ) $e:tt, )* ) => {{
        $(
            $set.spawn({
                let me = $self.clone();
                async move {
                    let ( $($skip,)* s, .. ) = &me.sdk;
                    $crate::SatoriSDK::start(s, &me).await
                }
            });
        )*
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! __satori_impl_start_app {
    ( ( $self:ident, $set:ident ), $( ( $($skip:tt)* ) $e:tt, )* ) => {{
        $(
            $set.spawn({
                let me = $self.clone();
                async move {
                    let ( $($skip,)* a, .. ) = &me.app;
                    $crate::SatoriApp::start(a, &me).await
                }
            });
        )*
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! __satori_impl_call_api {
    ( ( $self:ident, $bot:ident, $payload:ident ), $( ( $($skip:tt)* ) $e:tt, )* ) => {
        tokio::select!(
            $((s, true) = async { let ( $($skip,)* s, .. ) = &$self.sdk; (s, $crate::SatoriSDK::has_bot(s, $bot).await) } => $crate::SatoriSDK::call_api(s, $self, $bot, $payload).await,)*
            else => Err($crate::error::SatoriError::InvalidBot),
        )
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __satori_impl_handle_event {
    ( ( $self:ident, $event:ident ), $( ( $($skip:tt)* ) $e:tt, )* ) => {
        tokio::join!($(
            tokio::spawn({
                let me = $self.clone();
                let event = $event.clone();
                async move {
                    let ( $($skip,)* a, .. ) = &me.app;
                    $crate::SatoriApp::handle_event(a, &me, event).await
                }
            }),
        )*)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __satori_impl_get_logins {
    ( ( $self:ident, $result:ident ), $( ( $($skip:tt)* ) $e:tt, )* ) => {
        $(
            let ( $($skip,)* s, .. ) = &$self.sdk;
            $result.append(&mut $crate::SatoriSDK::get_logins(s).await);
        )*
    };
}

#[macro_export]
macro_rules! satori {
    {$vis:vis struct $name:ident { sdk: $s:tt, app: $a:tt, }} => {
        // $crate::__satori_assert_impl!($s: $crate::SatoriSDK);
        // $crate::__satori_assert_impl!($a: $crate::SatoriApp);

        $vis struct $name {
            sdk: $crate::__satori_wrap_tuple!($s),
            app: $crate::__satori_wrap_tuple!($a),
            stop: tokio_util::sync::CancellationToken,
            set: tokio::sync::RwLock<tokio::task::JoinSet<()>>,
        }

        impl $name {
            $vis fn new(sdk: $s, app: $a) -> std::sync::Arc<Self> {
                std::sync::Arc::new(Self {
                    sdk: $crate::__satori_convert_tuple!($s, sdk),
                    app: $crate::__satori_convert_tuple!($a, app),
                    stop: tokio_util::sync::CancellationToken::new(),
                    set: tokio::sync::RwLock::new(tokio::task::JoinSet::new()),
                })
            }
        }

        impl $crate::Satori for $name {
            async fn spawn(self: &std::sync::Arc<Self>) {
                tracing::info!(target: $crate::SATORI, "Starting...");
                let mut set = self.set.write().await;
                $crate::__satori_expand!(__satori_impl_start_sdk, (self, set), $s);
                $crate::__satori_expand!(__satori_impl_start_app, (self, set), $a);
            }

            async fn start(self: &std::sync::Arc<Self>) {
                $crate::Satori::spawn(self).await;
                while !self.set.read().await.is_empty() {
                    let _ = self.set.write().await.join_next().await; 
                }
            }

            async fn shutdown(self: &std::sync::Arc<Self>) {
                tracing::info!(target: $crate::SATORI, "Stopping...");
                self.stop.cancel();
                self.set.write().await.shutdown().await;
            }

            async fn call_api<T>(self: &std::sync::Arc<Self>, bot: &$crate::structs::BotId, payload: T) -> Result<serde_json::Value, $crate::error::SatoriError>
            where
                T: $crate::api::IntoRawApiCall + Send,
            {
                let payload = payload.into_raw();
                tracing::debug!(target: $crate::SATORI, ?bot, ?payload, "call api");
                $crate::__satori_expand!(__satori_impl_call_api, (self, bot, payload), $s)
            }

            async fn handle_event(self: &std::sync::Arc<Self>, event: $crate::structs::Event) {
                tracing::debug!(target: $crate::SATORI, ?event, "handle event");
                let _ = $crate::__satori_expand!(__satori_impl_handle_event, (self, event), $a);
            }

            async fn get_logins(self: &std::sync::Arc<Self>) -> Vec<$crate::structs::Login> {
                let mut result = vec![];
                $crate::__satori_expand!(__satori_impl_get_logins, (self, result), $s);
                result
            }

            async fn stopped(self: &std::sync::Arc<Self>) {
                self.stop.cancelled().await
            }
        }

        #[cfg(feature = "graceful-shutdown")]
        impl $name
        {
            $vis async fn start_with_graceful_shutdown(self: &std::sync::Arc<Self>, signal: impl std::future::Future) {
                $crate::Satori::spawn(self).await;
                tokio::select! {
                    _ = signal => $crate::Satori::shutdown(self).await,
                    _ = async { while !self.set.read().await.is_empty() {} } => {}
                }
            }
        }
    };
}
