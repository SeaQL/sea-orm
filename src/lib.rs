#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![deny(
    missing_debug_implementations,
    clippy::missing_panics_doc,
    clippy::unwrap_used,
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
//! [中文文档](https://github.com/SeaQL/sea-orm/blob/1.1.x/README-zh.md)
//!
//! #### SeaORM is a relational ORM to help you build web services in Rust with the familiarity of dynamic languages.
//!
//! [![GitHub stars](https://img.shields.io/github/stars/SeaQL/sea-orm.svg?style=social&label=Star&maxAge=1)](https://github.com/SeaQL/sea-orm/stargazers/)
//! If you like what we do, consider starring, sharing and contributing!
//!
//! Please help us with maintaining SeaORM by completing the [SeaQL Community Survey 2025](https://www.sea-ql.org/community-survey/)!
//!
//! [![Discord](https://img.shields.io/discord/873880840487206962?label=Discord)](https://discord.com/invite/uCPdDXzbdv)
//! Join our Discord server to chat with other members of the SeaQL community!
//!
//! ## Getting Started
//!
//! + [Documentation](https://www.sea-ql.org/SeaORM)
//! + [Tutorial](https://www.sea-ql.org/sea-orm-tutorial)
//! + [Cookbook](https://www.sea-ql.org/sea-orm-cookbook)
//!
//! Integration examples:
//!
//! + [Actix v4 Example](https://github.com/SeaQL/sea-orm/tree/master/examples/actix_example)
//! + [Axum Example](https://github.com/SeaQL/sea-orm/tree/master/examples/axum_example)
//! + [GraphQL Example](https://github.com/SeaQL/sea-orm/tree/master/examples/graphql_example)
//! + [jsonrpsee Example](https://github.com/SeaQL/sea-orm/tree/master/examples/jsonrpsee_example)
//! + [Loco TODO Example](https://github.com/SeaQL/sea-orm/tree/master/examples/loco_example) / [Loco REST Starter](https://github.com/SeaQL/sea-orm/tree/master/examples/loco_starter)
//! + [Poem Example](https://github.com/SeaQL/sea-orm/tree/master/examples/poem_example)
//! + [Rocket Example](https://github.com/SeaQL/sea-orm/tree/master/examples/rocket_example) / [Rocket OpenAPI Example](https://github.com/SeaQL/sea-orm/tree/master/examples/rocket_okapi_example)
//! + [Salvo Example](https://github.com/SeaQL/sea-orm/tree/master/examples/salvo_example)
//! + [Tonic Example](https://github.com/SeaQL/sea-orm/tree/master/examples/tonic_example)
//! + [Seaography Example](https://github.com/SeaQL/sea-orm/tree/master/examples/seaography_example)
//!
//! ## Features
//!
//! 1. Async
//!
//!     Relying on [SQLx](https://github.com/launchbadge/sqlx), SeaORM is a new library with async support from day 1.
//!
//! 2. Dynamic
//!
//!     Built upon [SeaQuery](https://github.com/SeaQL/sea-query), SeaORM allows you to build complex dynamic queries.
//!
//! 3. Service Oriented
//!
//!     Quickly build services that join, filter, sort and paginate data in REST, GraphQL and gRPC APIs.
//!
//! 4. Production Ready
//!
//!     SeaORM is feature-rich, well-tested and used in production by companies and startups.
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
//!
//! ### Nested Select
//!
//! ```
//! # use sea_orm::{DbConn, error::*, entity::*, query::*, tests_cfg::*};
//! # async fn function(db: &DbConn) -> Result<(), DbErr> {
//! use sea_orm::DerivePartialModel;
//!
//! #[derive(DerivePartialModel)]
//! #[sea_orm(entity = "cake::Entity", from_query_result)]
//! struct CakeWithFruit {
//!     id: i32,
//!     name: String,
//!     #[sea_orm(nested)]
//!     fruit: Option<Fruit>,
//! }
//!
//! #[derive(DerivePartialModel)]
//! #[sea_orm(entity = "fruit::Entity", from_query_result)]
//! struct Fruit {
//!     id: i32,
//!     name: String,
//! }
//!
//! let cakes: Vec<CakeWithFruit> = cake::Entity::find()
//!     .left_join(fruit::Entity)
//!     .into_partial_model()
//!     .all(db)
//!     .await?;
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
//! Fruit::insert_many([apple, pear]).exec(db).await?;
//! # Ok(())
//! # }
//! ```
//! ### Insert (advanced)
//! ```
//! # use sea_orm::{DbConn, TryInsertResult, error::*, entity::*, query::*, tests_cfg::*};
//! # async fn function_1(db: &DbConn) -> Result<(), DbErr> {
//! # let apple = fruit::ActiveModel {
//! #     name: Set("Apple".to_owned()),
//! #     ..Default::default() // no need to set primary key
//! # };
//! # let pear = fruit::ActiveModel {
//! #     name: Set("Pear".to_owned()),
//! #     ..Default::default()
//! # };
//! // insert many with returning (if supported by database)
//! let models: Vec<fruit::Model> = Fruit::insert_many([apple, pear])
//!     .exec_with_returning_many(db)
//!     .await?;
//! models[0]
//!     == fruit::Model {
//!         id: 1,
//!         name: "Apple".to_owned(),
//!         cake_id: None,
//!     };
//! # Ok(())
//! # }
//!
//! # async fn function_2(db: &DbConn) -> Result<(), DbErr> {
//! # let apple = fruit::ActiveModel {
//! #     name: Set("Apple".to_owned()),
//! #     ..Default::default() // no need to set primary key
//! # };
//! # let pear = fruit::ActiveModel {
//! #     name: Set("Pear".to_owned()),
//! #     ..Default::default()
//! # };
//! // insert with ON CONFLICT on primary key do nothing, with MySQL specific polyfill
//! let result = Fruit::insert_many([apple, pear])
//!     .on_conflict_do_nothing()
//!     .exec(db)
//!     .await?;
//!
//! matches!(result, TryInsertResult::Conflicted);
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
//! // delete many: DELETE FROM "fruit" WHERE "fruit"."name" LIKE '%Orange%'
//! fruit::Entity::delete_many()
//!     .filter(fruit::Column::Name.contains("Orange"))
//!     .exec(db)
//!     .await?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## 🧭 Seaography: instant GraphQL API
//!
//! [Seaography](https://github.com/SeaQL/seaography) is a GraphQL framework built on top of SeaORM. Seaography allows you to build GraphQL resolvers quickly. With just a few commands, you can launch a GraphQL server from SeaORM entities!
//!
//! Look at the [Seaography Example](https://github.com/SeaQL/sea-orm/tree/master/examples/seaography_example) to learn more.
//!
//! <img src="https://raw.githubusercontent.com/SeaQL/sea-orm/master/examples/seaography_example/Seaography%20example.png"/>
//!
//! ## 🖥️ SeaORM Pro: Effortless Admin Panel
//!
//! [SeaORM Pro](https://www.sea-ql.org/sea-orm-pro/) is an admin panel solution allowing you to quickly and easily launch an admin panel for your application - frontend development skills not required, but certainly nice to have!
//!
//! Features:
//!
//! + Full CRUD
//! + Built on React + GraphQL
//! + Built-in GraphQL resolver
//! + Customize the UI with simple TOML
//!
//! Learn More
//!
//! + [Example Repo](https://github.com/SeaQL/sea-orm-pro)
//! + [Getting Started with Loco](https://www.sea-ql.org/sea-orm-pro/docs/install-and-config/getting-started-loco/)
//! + [Getting Started with Axum](https://www.sea-ql.org/sea-orm-pro/docs/install-and-config/getting-started-axum/)
//!
//! ![](https://raw.githubusercontent.com/SeaQL/sea-orm/refs/heads/master/docs/sea-orm-pro-dark.png#gh-dark-mode-only)
//! ![](https://raw.githubusercontent.com/SeaQL/sea-orm/refs/heads/master/docs/sea-orm-pro-light.png#gh-light-mode-only)
//!
//! ## Releases
//!
//! [SeaORM 1.0](https://www.sea-ql.org/blog/2024-08-04-sea-orm-1.0/) is a stable release. The 1.x version will be updated until at least October 2025, and we'll decide whether to release a 2.0 version or extend the 1.x life cycle.
//!
//! It doesn't mean that SeaORM is 'done', we've designed an architecture to allow us to deliver new features without major breaking changes. In fact, more features are coming!
//!
//! + [Change Log](https://github.com/SeaQL/sea-orm/tree/master/CHANGELOG.md)
//!
//! ### Who's using SeaORM?
//!
//! Here is a short list of awesome open source software built with SeaORM. [Full list here](https://github.com/SeaQL/sea-orm/blob/master/COMMUNITY.md#built-with-seaorm). Feel free to submit yours!
//!
//! | Project | GitHub | Tagline |
//! |---------|--------|---------|
//! | [Zed](https://github.com/zed-industries/zed) | ![GitHub stars](https://img.shields.io/github/stars/zed-industries/zed.svg?style=social) | A high-performance, multiplayer code editor |
//! | [OpenObserve](https://github.com/openobserve/openobserve) | ![GitHub stars](https://img.shields.io/github/stars/openobserve/openobserve.svg?style=social) | Open-source observability platform |
//! | [RisingWave](https://github.com/risingwavelabs/risingwave) | ![GitHub stars](https://img.shields.io/github/stars/risingwavelabs/risingwave.svg?style=social) | Stream processing and management platform |
//! | [LLDAP](https://github.com/nitnelave/lldap) | ![GitHub stars](https://img.shields.io/github/stars/nitnelave/lldap.svg?style=social) | A light LDAP server for user management |
//! | [Warpgate](https://github.com/warp-tech/warpgate) | ![GitHub stars](https://img.shields.io/github/stars/warp-tech/warpgate.svg?style=social) | Smart SSH bastion that works with any SSH client |
//! | [Svix](https://github.com/svix/svix-webhooks) | ![GitHub stars](https://img.shields.io/github/stars/svix/svix-webhooks.svg?style=social) | The enterprise ready webhooks service |
//! | [Ryot](https://github.com/IgnisDa/ryot) | ![GitHub stars](https://img.shields.io/github/stars/ignisda/ryot.svg?style=social) | The only self hosted tracker you will ever need |
//! | [Lapdev](https://github.com/lapce/lapdev) | ![GitHub stars](https://img.shields.io/github/stars/lapce/lapdev.svg?style=social) | Self-hosted remote development enviroment |
//! | [System Initiative](https://github.com/systeminit/si) | ![GitHub stars](https://img.shields.io/github/stars/systeminit/si.svg?style=social) | DevOps Automation Platform |
//! | [OctoBase](https://github.com/toeverything/OctoBase) | ![GitHub stars](https://img.shields.io/github/stars/toeverything/OctoBase.svg?style=social) | A light-weight, scalable, offline collaborative data backend |
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
//!
//! We invite you to participate, contribute and together help build Rust's future.
//!
//! A big shout out to our contributors!
//!
//! [![Contributors](https://opencollective.com/sea-orm/contributors.svg?width=1000&button=false)](https://github.com/SeaQL/sea-orm/graphs/contributors)
//!
//! ## Sponsorship
//!
//! [SeaQL.org](https://www.sea-ql.org/) is an independent open-source organization run by passionate developers. If you enjoy using our libraries, please star and share our repositories. If you feel generous, a small donation via [GitHub Sponsor](https://github.com/sponsors/SeaQL) will be greatly appreciated, and goes a long way towards sustaining the organization.
//!
//! ### Gold Sponsors
//!
//! <table><tr>
//! <td><a href="https://qdx.co/">
//!   <img src="https://www.sea-ql.org/static/sponsors/QDX.svg" width="138"/>
//! </a></td>
//! </tr></table>
//!
//! [QDX](https://qdx.co/) pioneers quantum dynamics-powered drug discovery, leveraging AI and supercomputing to accelerate molecular modeling.
//! We're immensely grateful to QDX for sponsoring the development of SeaORM, the SQL toolkit that powers their data engineering workflows.
//!
//! ### Silver Sponsors
//!
//! We’re grateful to our silver sponsors: Digital Ocean, for sponsoring our servers. And JetBrains, for sponsoring our IDE.
//!
//! <table><tr>
//! <td><a href="https://www.digitalocean.com/">
//!   <img src="https://www.sea-ql.org/static/sponsors/DigitalOcean.svg" width="125">
//! </a></td>
//!
//! <td><a href="https://www.jetbrains.com/">
//!   <img src="https://www.sea-ql.org/static/sponsors/JetBrains.svg" width="125">
//! </a></td>
//! </tr></table>
//!
//! ## Mascot
//!
//! A friend of Ferris, Terres the hermit crab is the official mascot of SeaORM. His hobby is collecting shells.
//!
//! <img alt="Terres" src="https://www.sea-ql.org/SeaORM/img/Terres.png" width="400"/>
//!
//! ### Rustacean Sticker Pack 🦀
//!
//! The Rustacean Sticker Pack is the perfect way to express your passion for Rust.
//! Our stickers are made with a premium water-resistant vinyl with a unique matte finish.
//! Stick them on your laptop, notebook, or any gadget to show off your love for Rust!
//!
//! Sticker Pack Contents:
//! - Logo of SeaQL projects: SeaQL, SeaORM, SeaQuery, Seaography, FireDBG
//! - Mascot of SeaQL: Terres the Hermit Crab
//! - Mascot of Rust: Ferris the Crab
//! - The Rustacean word
//!
//! [Support SeaQL and get a Sticker Pack!](https://www.sea-ql.org/sticker-pack/) All proceeds contributes directly to the ongoing development of SeaQL projects.
//!
//! <a href="https://www.sea-ql.org/sticker-pack/"><img alt="Rustacean Sticker Pack by SeaQL" src="https://www.sea-ql.org/static/sticker-pack-1s.jpg" width="600"/></a>
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
/// Types and methods to perform metric collection
pub mod metric;
/// Types and methods to perform queries
pub mod query;
/// Types that defines the schemas of an Entity
pub mod schema;
/// Helpers for working with Value
pub mod value;

#[doc(hidden)]
#[cfg(all(feature = "macros", feature = "tests-cfg"))]
pub mod tests_cfg;
mod util;

pub use database::*;
#[allow(unused_imports)]
pub use driver::*;
pub use entity::*;
pub use error::*;
pub use executor::*;
pub use query::*;
pub use schema::*;

#[cfg(feature = "macros")]
pub use sea_orm_macros::{
    DeriveActiveEnum, DeriveActiveModel, DeriveActiveModelBehavior, DeriveColumn,
    DeriveCustomColumn, DeriveDisplay, DeriveEntity, DeriveEntityModel, DeriveIden,
    DeriveIntoActiveModel, DeriveMigrationName, DeriveModel, DerivePartialModel, DerivePrimaryKey,
    DeriveRelatedEntity, DeriveRelation, DeriveValueType, FromJsonQueryResult, FromQueryResult,
};

pub use sea_query;
pub use sea_query::Iden;

pub use sea_orm_macros::EnumIter;
pub use strum;

#[cfg(feature = "sqlx-dep")]
pub use sqlx;
