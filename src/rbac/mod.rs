//! Role-based access control for SeaORM (`rbac` feature).
//!
//! Authorises connections at the SQL level: the [`RbacEngine`] reads
//! permissions out of dedicated database tables, attaches them to a
//! [`DatabaseConnection`](crate::DatabaseConnection) (producing a
//! [`RestrictedConnection`](crate::RestrictedConnection)), and then every
//! query / mutation is checked against the user's roles before being sent
//! to the database.
//!
//! See the [Role Based Access Control](https://www.sea-ql.org/blog/2025-09-30-sea-orm-rbac/)
//! blog post for an end-to-end walkthrough.

#![allow(missing_docs)]

mod engine;
pub use engine::*;

pub mod entity;
pub use entity::user::UserId as RbacUserId;

pub mod context;
pub use context::*;

mod error;
pub use error::Error as RbacError;
use error::*;

pub mod schema;

/// Wildcard token (`"*"`) accepted by RBAC tables to mean "any permission"
/// or "any resource".
pub const WILDCARD: &str = "*";

pub use sea_query::audit::{AccessType, SchemaOper};
