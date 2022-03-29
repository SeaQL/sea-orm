#[cfg(feature = "mock")]
mod mock;

#[cfg(feature = "mock")]
pub use mock::*;

pub use sea_connection::database::*;
