//! <div align="center">
//!
//!   <img src="docs/SeaORM banner.png"/>
//!
//!   <h1>SeaORM</h1>
//!
//!   <p>
//!     <strong>🐚 An async & dynamic ORM for Rust</strong>
//!   </p>
//!
//!   <sub>Built with 🔥 by 🌊🦀🐚</sub>
//!
//! </div>
//!
//! # SeaORM
//!
//! Inspired by ActiveRecord, Eloquent and TypeORM, SeaORM aims to provide you an intuitive and ergonomic
//! API to make working with databases in Rust a first-class experience.
//!
//! ```ignore
//! This is an early WIP of SeaORM, and is not yet published. See [example](examples/sqlx-mysql/src) for demo usage.
//! ```
//!
//! ## Features
//!
//! 1. Async
//!
//! Relying on SQLx, SeaORM is a new library with async support from day 1.
//!
//! 2. Dynamic
//!
//! Built upon SeaQuery, a dynamic query builder, SeaORM allows you to build complex queries without 'fighting the ORM'.
//!
//! 3. Testable
//!
//! Use mock connections to write unit tests for your logic.
//!
//! 4. API oriented
//!
//! Quickly build search models that help you join, filter, sort and paginate data in APIs.
//!
//! # A quick taste of SeaORM
//!
//! ## Select
//! ```
//! # use sea_orm::{DbConn, entity::*, query::*, tests_cfg::*};
//! # async fn function(db: &DbConn) -> Result<(), QueryErr> {
//! #
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
//! let cake_with_fruits: Vec<(cake::Model, Vec<fruit::Model>)> = Cake::find()
//!     .find_with_related(Fruit)
//!     .all(db)
//!     .await?;
//!
//! # Ok(())
//! # }
//! ```
//! ## Insert
//! ```
//! # use sea_orm::{DbConn, entity::*, query::*, tests_cfg::*};
//! # async fn function(db: &DbConn) -> Result<(), ExecErr> {
//! #
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
//! let res: InsertResult = Fruit::insert(pear).exec(db).await?;
//!
//! println!("InsertResult: {}", res.last_insert_id);
//! #
//! # Ok(())
//! # }
//! #
//! # async fn function2(db: &DbConn) -> Result<(), ExecErr> {
//! # let apple = fruit::ActiveModel {
//! #     name: Set("Apple".to_owned()),
//! #     ..Default::default() // no need to set primary key
//! # };
//! #
//! # let pear = fruit::ActiveModel {
//! #     name: Set("Pear".to_owned()),
//! #     ..Default::default()
//! # };
//!
//! // insert many
//! Fruit::insert_many(vec![apple, pear]).exec(db).await?;
//! #
//! # Ok(())
//! # }
//! ```
//! ## Update
//! ```
//! # use sea_orm::{DbConn, entity::*, query::*, tests_cfg::*};
//! #
//! use sea_orm::sea_query::{Expr, Value};
//!
//! # async fn function(db: &DbConn) -> Result<(), QueryErr> {
//! let pear: Option<fruit::Model> = Fruit::find_by_id(1).one(db).await?;
//! # Ok(())
//! # }
//! #
//! # async fn function2(db: &DbConn) -> Result<(), ExecErr> {
//! # let pear: Option<fruit::Model> = Fruit::find_by_id(1).one(db).await.unwrap();
//!
//! let mut pear: fruit::ActiveModel = pear.unwrap().into();
//! pear.name = Set("Sweet pear".to_owned());
//!
//! // update one
//! let pear: fruit::ActiveModel = Fruit::update(pear).exec(db).await?;
//!
//! // update many: UPDATE "fruit" SET "cake_id" = NULL WHERE "fruit"."name" LIKE '%Apple%'
//! Fruit::update_many()
//!     .col_expr(fruit::Column::CakeId, Expr::value(Value::Null))
//!     .filter(fruit::Column::Name.contains("Apple"))
//!     .exec(db)
//!     .await?;
//!
//! # Ok(())
//! # }
//! ```
//! ## Save
//! ```
//! # use sea_orm::{DbConn, entity::*, query::*, tests_cfg::*};
//! #
//! # async fn function(db: &DbConn) -> Result<(), ExecErr> {
//! let banana = fruit::ActiveModel {
//!     id: Unset(None),
//!     name: Set("Banana".to_owned()),
//!     ..Default::default()
//! };
//!
//! // create, because primary key `id` is `Unset`
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
//! ## Delete
//! ```
//! # use sea_orm::{DbConn, entity::*, query::*, tests_cfg::*};
//! #
//! # async fn function(db: &DbConn) -> Result<(), QueryErr> {
//! let orange: Option<fruit::Model> = Fruit::find_by_id(1).one(db).await?;
//! # Ok(())
//! # }
//! #
//! # async fn function2(db: &DbConn) -> Result<(), ExecErr> {
//! # let orange: Option<fruit::Model> = Fruit::find_by_id(1).one(db).await.unwrap();
//! let orange: fruit::ActiveModel = orange.unwrap().into();
//!
//! // delete one
//! fruit::Entity::delete(orange).exec(db).await?;
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
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/SeaQL/sea-query/master/docs/SeaQL icon dark.png"
)]

mod database;
mod driver;
pub mod entity;
mod executor;
pub mod query;
#[doc(hidden)]
pub mod tests_cfg;
mod util;

pub use database::*;
pub use driver::*;
pub use entity::*;
pub use executor::*;
pub use query::*;

pub use sea_orm_macros::{
    DeriveActiveModel, DeriveActiveModelBehavior, DeriveColumn, DeriveEntity, DeriveModel,
    DerivePrimaryKey, FromQueryResult,
};
pub use sea_query;
pub use sea_query::Iden;
pub use strum::EnumIter;
