use std::fmt::Display;

use colored::Colorize;

#[cfg(feature = "codegen")]
pub mod generate;
#[cfg(feature = "cli")]
pub mod entity;
pub mod migrate;
pub mod subprocess;

#[cfg(feature = "codegen")]
pub use generate::*;
pub use migrate::*;

pub fn handle_error<E>(error: E)
where
    E: Display,
{
    eprintln!("{} {error}", "Error:".red().bold());
    ::std::process::exit(1);
}
