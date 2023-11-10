pub const SATORI: &str = "Satori";

mod core;
pub use core::*;

mod macros;

pub mod api;
pub mod error;
pub mod impls;
pub mod structs;

#[cfg(feature = "message")]
pub mod message;
