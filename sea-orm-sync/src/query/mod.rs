//! Query builders used to express SELECT, INSERT, UPDATE, and DELETE
//! statements against an [`Entity`](crate::EntityTrait).
//!
//! Most queries originate from an entity:
//!
//! ```ignore
//! cake::Entity::find()        // -> Select<cake::Entity>
//! cake::Entity::insert(am)    // -> Insert<cake::ActiveModel>
//! cake::Entity::update(am)    // -> UpdateOne<cake::ActiveModel>
//! cake::Entity::delete(am)    // -> DeleteOne<cake::Entity>
//! ```
//!
//! The builders implement composable traits — [`QueryFilter`],
//! [`QuerySelect`], [`QueryOrder`], [`QueryTrait`] — for adding `WHERE`
//! clauses, joins, ordering, and column projections. Once composed, hand the
//! query to a [`ConnectionTrait`] via `.one(db)`, `.all(db)`, `.exec(db)`,
//! `.stream(db)`, etc.
//!
//! For raw SQL, use [`Statement`] together with the [`raw_sql!`](crate::raw_sql)
//! macro.

pub(crate) mod combine;
mod debug;
mod delete;
mod helper;
mod insert;
mod join;
#[cfg(feature = "with-json")]
mod json;
mod loader;
mod select;
mod traits;
mod update;
mod util;

pub use combine::{SelectA, SelectB, SelectC};
pub use debug::*;
pub use delete::*;
pub use helper::*;
pub use insert::*;
#[cfg(feature = "with-json")]
pub use json::*;
pub use loader::*;
pub use select::*;
pub use traits::*;
pub use update::*;
pub(crate) use util::*;

pub use crate::{
    ConnectionTrait, CountTrait, CursorTrait, InsertResult, PaginatorTrait, SelectExt, Statement,
    TransactionTrait, UpdateResult, Value, Values,
};
pub use sea_query::ExprTrait;

#[cfg(feature = "stream")]
pub use crate::StreamTrait;
