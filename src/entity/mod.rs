//! Entities and the types and traits that describe them.
//!
//! In SeaORM, every database table is represented by an **Entity** — a unit
//! struct implementing [`EntityTrait`]. The Entity ties together:
//!
//! - the table's **name** (via [`EntityName`]),
//! - its **columns** (via [`ColumnTrait`] and the strongly-typed `COLUMN` constant),
//! - its **primary key** (via [`PrimaryKeyTrait`] / [`PrimaryKeyToColumn`]),
//! - its **relations** to other entities (via [`RelationTrait`]; in 2.0,
//!   relations can also be declared directly on the `Model` struct as
//!   [`HasOne`](compound::HasOne) / [`HasMany`](compound::HasMany) fields).
//!
//! Each Entity has two companion types:
//!
//! - **[`Model`](ModelTrait)** — a plain struct mirroring a row of the table,
//!   used for reads.
//! - **[`ActiveModel`](ActiveModelTrait)** — a struct where every field is
//!   wrapped in [`ActiveValue`], used for inserts and partial updates.
//!
//! From an Entity you can build select, insert, update, and delete queries
//! via [`EntityTrait::find`], [`EntityTrait::insert`], [`EntityTrait::update`],
//! and [`EntityTrait::delete`] (plus their `_many` / `_by_id` siblings).
//!
//! # Defining an Entity (2.0 dense format)
//!
//! The recommended way to define an entity in SeaORM 2.0 is the dense entity
//! format, where the relations live directly on the `Model` struct as
//! [`HasOne`](compound::HasOne) / [`HasMany`](compound::HasMany) fields:
//!
//! ```
//! # #[cfg(feature = "macros")]
//! # mod entities {
//! # mod fruit {
//! #     use sea_orm::entity::prelude::*;
//! #     #[sea_orm::model]
//! #     #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
//! #     #[sea_orm(table_name = "fruit")]
//! #     pub struct Model {
//! #         #[sea_orm(primary_key)]
//! #         pub id: i32,
//! #         pub cake_id: Option<i32>,
//! #         #[sea_orm(belongs_to, from = "cake_id", to = "id")]
//! #         pub cake: HasOne<super::cake::Entity>,
//! #     }
//! #     impl ActiveModelBehavior for ActiveModel {}
//! # }
//! mod cake {
//!     use sea_orm::entity::prelude::*;
//!
//!     #[sea_orm::model]
//!     #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
//!     #[sea_orm(table_name = "cake")]
//!     pub struct Model {
//!         #[sea_orm(primary_key)]
//!         pub id: i32,
//!         pub name: String,
//!         #[sea_orm(has_many)]
//!         pub fruits: HasMany<super::fruit::Entity>,
//!     }
//!
//!     impl ActiveModelBehavior for ActiveModel {}
//! }
//! # }
//! ```
//!
//! # Defining an Entity (1.0 compact format)
//!
//! The 1.0 compact format remains supported. Here relations are declared as a
//! separate `Relation` enum implementing [`RelationTrait`], plus an explicit
//! [`Related`] impl per foreign entity:
//!
//! ```
//! # #[cfg(feature = "macros")]
//! # mod entities {
//! # mod fruit {
//! #     use sea_orm::entity::prelude::*;
//! #     #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
//! #     #[sea_orm(table_name = "fruit")]
//! #     pub struct Model {
//! #         #[sea_orm(primary_key)]
//! #         pub id: i32,
//! #         pub cake_id: Option<i32>,
//! #     }
//! #     #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
//! #     pub enum Relation {
//! #         #[sea_orm(
//! #             belongs_to = "super::cake::Entity",
//! #             from = "Column::CakeId",
//! #             to = "super::cake::Column::Id"
//! #         )]
//! #         Cake,
//! #     }
//! #     impl Related<super::cake::Entity> for Entity {
//! #         fn to() -> RelationDef { Relation::Cake.def() }
//! #     }
//! #     impl ActiveModelBehavior for ActiveModel {}
//! # }
//! mod cake {
//!     use sea_orm::entity::prelude::*;
//!
//!     #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
//!     #[sea_orm(table_name = "cake")]
//!     pub struct Model {
//!         #[sea_orm(primary_key)]
//!         pub id: i32,
//!         pub name: String,
//!     }
//!
//!     #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
//!     pub enum Relation {
//!         #[sea_orm(has_many = "super::fruit::Entity")]
//!         Fruit,
//!     }
//!
//!     impl Related<super::fruit::Entity> for Entity {
//!         fn to() -> RelationDef {
//!             Relation::Fruit.def()
//!         }
//!     }
//!
//!     impl ActiveModelBehavior for ActiveModel {}
//! }
//! # }
//! ```
//!
//! Entity files are usually generated for you with `sea-orm-cli` against an
//! existing database. See the [crate-level documentation](crate) for a
//! walkthrough of the most common operations.
mod active_enum;
mod active_model;
mod active_model_ex;
mod active_value;
#[cfg(feature = "with-arrow")]
mod arrow_schema;
mod base_entity;
pub(crate) mod column;
mod column_def;
pub mod compound;
mod identity;
mod link;
mod model;
mod partial_model;
/// Re-exports the types and traits most commonly needed to define and use
/// entities. Glob-import this module in entity files: `use sea_orm::entity::prelude::*;`.
pub mod prelude;
mod primary_key;
#[cfg(feature = "entity-registry")]
mod registry;
mod relation;
#[cfg(feature = "with-arrow")]
pub(crate) mod with_arrow;

pub use active_enum::*;
pub use active_model::*;
pub use active_model_ex::*;
pub use active_value::*;
#[cfg(feature = "with-arrow")]
pub use arrow_schema::*;
pub use base_entity::*;
pub use column::*;
pub use column_def::*;
pub use compound::EntityLoaderTrait;
pub use identity::*;
pub use link::*;
pub use model::*;
pub use partial_model::*;
pub use primary_key::*;
#[cfg(feature = "entity-registry")]
pub use registry::*;
pub use relation::*;
