#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![deny(
    missing_debug_implementations,
    clippy::print_stderr,
    clippy::print_stdout
)]

//! <div align="center">
//!
//!   <img src="https://www.sea-ql.org/SeaORM/img/SeaORM banner.png"/>
//!
//!   <h1>SeaORM</h1>
//!
//!   <h3>🐚 An async & dynamic ORM for Rust</h3>
//!
//!   [![crate](https://img.shields.io/crates/v/sea-orm.svg)](https://crates.io/crates/sea-orm)
//!   [![docs](https://docs.rs/sea-orm/badge.svg)](https://docs.rs/sea-orm)
//!   [![build status](https://github.com/SeaQL/sea-orm/actions/workflows/rust.yml/badge.svg)](https://github.com/SeaQL/sea-orm/actions/workflows/rust.yml)
//!
//! </div>
//!
//! # SeaORM
//!
//! #### SeaORM is a relational ORM to help you build web services in Rust with the familiarity of dynamic languages.
//!
//! ## Getting Started
//!
//! [![GitHub stars](https://img.shields.io/github/stars/SeaQL/sea-orm.svg?style=social&label=Star&maxAge=1)](https://github.com/SeaQL/sea-orm/stargazers/)
//! If you like what we do, consider starring, commenting, sharing and contributing!
//!
//! [![Discord](https://img.shields.io/discord/873880840487206962?label=Discord)](https://discord.com/invite/uCPdDXzbdv)
//! Join our Discord server to chat with others in the SeaQL community!
//!
//! + [Getting Started](https://www.sea-ql.org/SeaORM/docs/index)
//! + [Usage Example](https://github.com/SeaQL/sea-orm/tree/master/examples/basic)
//!
//! Integration examples
//!
//! + [Rocket Example](https://github.com/SeaQL/sea-orm/tree/master/examples/rocket_example)
//! + [Actix Example](https://github.com/SeaQL/sea-orm/tree/master/examples/actix_example)
//! + [Axum Example](https://github.com/SeaQL/sea-orm/tree/master/examples/axum_example)
//! + [Poem Example](https://github.com/SeaQL/sea-orm/tree/master/examples/poem_example)
//! + [GraphQL Example](https://github.com/SeaQL/sea-orm/tree/master/examples/graphql_example)
//! + [jsonrpsee Example](https://github.com/SeaQL/sea-orm/tree/master/examples/jsonrpsee_example)
//! + [Tonic Example](https://github.com/SeaQL/sea-orm/tree/master/examples/tonic_example)
//!
//! ## Features
//!
//! 1. Async
//!
//!     Relying on [SQLx](https://github.com/launchbadge/sqlx), SeaORM is a new library with async support from day 1.
//!
//! 2. Dynamic
//!
//!     Built upon [SeaQuery](https://github.com/SeaQL/sea-query), SeaORM allows you to build complex queries without 'fighting the ORM'.
//!
//! 3. Testable
//!
//!     Use mock connections to write unit tests for your logic.
//!
//! 4. Service Oriented
//!
//!     Quickly build services that join, filter, sort and paginate data in APIs.
//!
//! ## A quick taste of SeaORM
//!
//! ### Entity
//! ```
//! # #[cfg(feature = "macros")]
//! # mod entities {
//! # mod fruit {
//! # use sea_orm::entity::prelude::*;
//! # #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
//! # #[sea_orm(table_name = "fruit")]
//! # pub struct Model {
//! #     #[sea_orm(primary_key)]
//! #     pub id: i32,
//! #     pub name: String,
//! #     pub cake_id: Option<i32>,
//! # }
//! # #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
//! # pub enum Relation {
//! #     #[sea_orm(
//! #         belongs_to = "super::cake::Entity",
//! #         from = "Column::CakeId",
//! #         to = "super::cake::Column::Id"
//! #     )]
//! #     Cake,
//! # }
//! # impl Related<super::cake::Entity> for Entity {
//! #     fn to() -> RelationDef {
//! #         Relation::Cake.def()
//! #     }
//! # }
//! # impl ActiveModelBehavior for ActiveModel {}
//! # }
//! # mod cake {
//! use sea_orm::entity::prelude::*;
//!
//! #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
//! #[sea_orm(table_name = "cake")]
//! pub struct Model {
//!     #[sea_orm(primary_key)]
//!     pub id: i32,
//!     pub name: String,
//! }
//!
//! #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
//! pub enum Relation {
//!     #[sea_orm(has_many = "super::fruit::Entity")]
//!     Fruit,
//! }
//!
//! impl Related<super::fruit::Entity> for Entity {
//!     fn to() -> RelationDef {
//!         Relation::Fruit.def()
//!     }
//! }
//! # impl ActiveModelBehavior for ActiveModel {}
//! # }
//! # }
//! ```
//!
//! ### Select
//! ```
//! # use sea_orm::{DbConn, error::*, entity::*, query::*, tests_cfg::*};
//! # async fn function(db: &DbConn) -> Result<(), DbErr> {
//! // find all models
//! let cakes: Vec<cake::Model> = Cake::find().all(db).await?;
//!
//! // find and filter
//! let chocolate: Vec<cake::Model> = Cake::find()
//!     .filter(cake::Column::Name.contains("chocolate"))
//!     .all(db)
//!     .await?;
//!
//! // find one model
//! let cheese: Option<cake::Model> = Cake::find_by_id(1).one(db).await?;
//! let cheese: cake::Model = cheese.unwrap();
//!
//! // find related models (lazy)
//! let fruits: Vec<fruit::Model> = cheese.find_related(Fruit).all(db).await?;
//!
//! // find related models (eager)
//! let cake_with_fruits: Vec<(cake::Model, Vec<fruit::Model>)> =
//!     Cake::find().find_with_related(Fruit).all(db).await?;
//!
//! # Ok(())
//! # }
//! ```
//! ### Insert
//! ```
//! # use sea_orm::{DbConn, error::*, entity::*, query::*, tests_cfg::*};
//! # async fn function(db: &DbConn) -> Result<(), DbErr> {
//! let apple = fruit::ActiveModel {
//!     name: Set("Apple".to_owned()),
//!     ..Default::default() // no need to set primary key
//! };
//!
//! let pear = fruit::ActiveModel {
//!     name: Set("Pear".to_owned()),
//!     ..Default::default()
//! };
//!
//! // insert one
//! let pear = pear.insert(db).await?;
//! # Ok(())
//! # }
//! # async fn function2(db: &DbConn) -> Result<(), DbErr> {
//! # let apple = fruit::ActiveModel {
//! #     name: Set("Apple".to_owned()),
//! #     ..Default::default() // no need to set primary key
//! # };
//! # let pear = fruit::ActiveModel {
//! #     name: Set("Pear".to_owned()),
//! #     ..Default::default()
//! # };
//!
//! // insert many
//! Fruit::insert_many(vec![apple, pear]).exec(db).await?;
//! # Ok(())
//! # }
//! ```
//! ### Update
//! ```
//! # use sea_orm::{DbConn, error::*, entity::*, query::*, tests_cfg::*};
//! use sea_orm::sea_query::{Expr, Value};
//!
//! # async fn function(db: &DbConn) -> Result<(), DbErr> {
//! let pear: Option<fruit::Model> = Fruit::find_by_id(1).one(db).await?;
//! let mut pear: fruit::ActiveModel = pear.unwrap().into();
//!
//! pear.name = Set("Sweet pear".to_owned());
//!
//! // update one
//! let pear: fruit::Model = pear.update(db).await?;
//!
//! // update many: UPDATE "fruit" SET "cake_id" = NULL WHERE "fruit"."name" LIKE '%Apple%'
//! Fruit::update_many()
//!     .col_expr(fruit::Column::CakeId, Expr::value(Value::Int(None)))
//!     .filter(fruit::Column::Name.contains("Apple"))
//!     .exec(db)
//!     .await?;
//!
//! # Ok(())
//! # }
//! ```
//! ### Save
//! ```
//! # use sea_orm::{DbConn, error::*, entity::*, query::*, tests_cfg::*};
//! # async fn function(db: &DbConn) -> Result<(), DbErr> {
//! let banana = fruit::ActiveModel {
//!     id: NotSet,
//!     name: Set("Banana".to_owned()),
//!     ..Default::default()
//! };
//!
//! // create, because primary key `id` is `NotSet`
//! let mut banana = banana.save(db).await?;
//!
//! banana.name = Set("Banana Mongo".to_owned());
//!
//! // update, because primary key `id` is `Set`
//! let banana = banana.save(db).await?;
//!
//! # Ok(())
//! # }
//! ```
//! ### Delete
//! ```
//! # use sea_orm::{DbConn, error::*, entity::*, query::*, tests_cfg::*};
//! # async fn function(db: &DbConn) -> Result<(), DbErr> {
//! // delete one
//! let orange: Option<fruit::Model> = Fruit::find_by_id(1).one(db).await?;
//! let orange: fruit::Model = orange.unwrap();
//! fruit::Entity::delete(orange.into_active_model())
//!     .exec(db)
//!     .await?;
//!
//! // or simply
//! let orange: Option<fruit::Model> = Fruit::find_by_id(1).one(db).await?;
//! let orange: fruit::Model = orange.unwrap();
//! orange.delete(db).await?;
//!
//! // delete many: DELETE FROM "fruit" WHERE "fruit"."name" LIKE 'Orange'
//! fruit::Entity::delete_many()
//!     .filter(fruit::Column::Name.contains("Orange"))
//!     .exec(db)
//!     .await?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Learn More
//!
//! 1. [Design](https://github.com/SeaQL/sea-orm/tree/master/DESIGN.md)
//! 1. [Architecture](https://github.com/SeaQL/sea-orm/tree/master/ARCHITECTURE.md)
//! 1. [Release Model](https://www.sea-ql.org/SeaORM/blog/2021-08-30-release-model)
//! 1. [Change Log](https://github.com/SeaQL/sea-orm/tree/master/CHANGELOG.md)
//!
//! ## Who's using SeaORM?
//!
//! SeaORM is the foundation of [StarfishQL](https://github.com/SeaQL/starfish-ql), an experimental graph database and query engine.
//!
//! For more projects, see [Built with SeaORM](https://github.com/SeaQL/sea-orm/blob/master/COMMUNITY.md#built-with-seaorm).
//!
//! ## License
//!
//! Licensed under either of
//!
//! -   Apache License, Version 2.0
//!     ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
//! -   MIT license
//!     ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
//!
//! at your option.
//!
//! ## Contribution
//!
//! Unless you explicitly state otherwise, any contribution intentionally submitted
//! for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
//! dual licensed as above, without any additional terms or conditions.
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/SeaQL/sea-query/master/docs/SeaQL icon dark.png"
)]

mod database;
mod docs;
mod driver;
/// Module for the Entity type and operations
pub mod entity;
/// Error types for all database operations
pub mod error;
/// This module performs execution of queries on a Model or ActiveModel
mod executor;
/// Holds types and methods to perform metric collection
pub mod metric;
/// Holds types and methods to perform queries
pub mod query;
/// Holds types that defines the schemas of an Entity
pub mod schema;
#[doc(hidden)]
#[cfg(feature = "macros")]
pub mod tests_cfg;
mod util;

pub use database::*;
pub use driver::*;
pub use entity::*;
pub use error::*;
pub use executor::*;
pub use query::*;
pub use schema::*;

#[cfg(feature = "macros")]
pub use sea_orm_macros::{
    DeriveActiveEnum, DeriveActiveModel, DeriveActiveModelBehavior, DeriveColumn,
    DeriveCustomColumn, DeriveEntity, DeriveEntityModel, DeriveIntoActiveModel, DeriveModel,
    DerivePrimaryKey, DeriveRelation, FromQueryResult,
};

pub use sea_query;
pub use sea_query::Iden;
#[cfg(feature = "macros")]
pub use sea_query::Iden as DeriveIden;

pub use strum;
pub use strum::EnumIter;
