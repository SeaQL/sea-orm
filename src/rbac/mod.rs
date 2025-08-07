#![allow(missing_docs)]

mod engine;
pub use engine::*;

pub mod entity;
pub use entity::user::UserId as RbacUserId;

mod error;
pub use error::Error as RbacError;
use error::*;

pub mod schema;

/// This could be used to denote any permission or any resources.
pub const WILDCARD: &str = "*";

pub use sea_query::audit::{AccessType, SchemaOper};
