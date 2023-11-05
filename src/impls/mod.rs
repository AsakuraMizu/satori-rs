#[cfg(any(feature = "net-app", feature = "net-sdk"))]
pub mod net;

#[cfg(feature = "onebot11")]
pub mod onebot11;

mod arc;
mod tuple;
