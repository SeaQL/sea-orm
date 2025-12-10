use std::fmt::Display;

#[cfg(feature = "codegen")]
pub mod config;
pub mod generate;
pub mod migrate;

#[cfg(feature = "codegen")]
pub use config::*;
pub use generate::*;
pub use migrate::*;

pub fn handle_error<E>(error: E)
where
    E: Display,
{
    eprintln!("{error}");
    ::std::process::exit(1);
}
