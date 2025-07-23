#![allow(missing_docs)]

pub mod engine;
pub mod entity;
mod error;

pub use error::Error as RbacError;
use error::*;

pub const WILDCARD: &str = "*";
