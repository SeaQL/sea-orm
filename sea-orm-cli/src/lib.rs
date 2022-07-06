#[macro_use]
extern crate simple_log;

pub mod cli;
#[cfg(feature = "codegen")]
pub mod commands;

pub use cli::*;
#[cfg(feature = "codegen")]
pub use commands::*;
