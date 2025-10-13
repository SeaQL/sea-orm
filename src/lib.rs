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
//! [中文文档](https://github.com/SeaQL/sea-orm/blob/master/README-zh.md)
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
//! + [Seaography Example (Bakery)](https://github.com/SeaQL/sea-orm/tree/master/examples/seaography_example) / [Seaography Example (Sakila)](https://github.com/SeaQL/seaography/tree/main/examples/sqlite)
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
//! Let's have a quick walk through of the unique features of SeaORM.
//!
//! ### Entity
//! You don't have to write this by hand! Entity files can be generated from an existing database with `sea-orm-cli`.
//! ```
//! # #[cfg(feature = "macros")]
//! # mod entities {
//! # mod fruit {
//! # use sea_orm::entity::prelude::*;
//! # #[sea_orm::model]
//! # #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
//! # #[sea_orm(table_name = "fruit")]
//! # pub struct Model {
//! #     #[sea_orm(primary_key)]
//! #     pub id: i32,
//! #     pub name: String,
//! #     pub cake_id: Option<i32>,
//! #     #[sea_orm(relation, from = "CakeId", to = "Id")]
//! #     pub cake: BelongsTo<super::cake::Entity>,
//! # }
//! # impl ActiveModelBehavior for ActiveModel {}
//! # }
//! # mod filling {
//! # use sea_orm::entity::prelude::*;
//! # #[sea_orm::model]
//! # #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
//! # #[sea_orm(table_name = "filling")]
//! # pub struct Model {
//! #     #[sea_orm(primary_key)]
//! #     pub id: i32,
//! #     pub name: String,
//! #     #[sea_orm(relation, via = "cake_filling")]
//! #     pub cakes: HasMany<super::cake::Entity>,
//! # }
//! # impl ActiveModelBehavior for ActiveModel {}
//! # }
//! # mod cake_filling {
//! # use sea_orm::entity::prelude::*;
//! # #[sea_orm::model]
//! # #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
//! # #[sea_orm(table_name = "cake_filling")]
//! # pub struct Model {
//! #     #[sea_orm(primary_key, auto_increment = false)]
//! #     pub cake_id: i32,
//! #     #[sea_orm(primary_key, auto_increment = false)]
//! #     pub filling_id: i32,
//! #     #[sea_orm(relation, from = "CakeId", to = "Id")]
//! #     pub cake: BelongsTo<super::cake::Entity> ,
//! #     #[sea_orm(relation, from = "FillingId", to = "Id")]
//! #     pub filling: BelongsTo<super::filling::Entity> ,
//! # }
//! # impl ActiveModelBehavior for ActiveModel {}
//! # }
//! # mod cake {
//! use sea_orm::entity::prelude::*;
//!
//! #[sea_orm::model]
//! #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
//! #[sea_orm(table_name = "cake")]
//! pub struct Model {
//!     #[sea_orm(primary_key)]
//!     pub id: i32,
//!     pub name: String,
//!     #[sea_orm(relation)]
//!     pub fruit: HasOne<super::fruit::Entity>,
//!     #[sea_orm(relation, via = "cake_filling")]
//!     pub fillings: HasMany<super::filling::Entity>,
//! }
//! # impl ActiveModelBehavior for ActiveModel {}
//! # }
//! # }
//! ```
//! ### Entity Loader
//!
//! It's a breeze to work with nested data structures.
//! The Entity Loader intelligently uses join for 1-1 and data loader for 1-N relations.
//! ```
//! # use sea_orm::{DbConn, error::*, prelude::*, entity::*, query::*, tests_cfg::*};
//! # async fn function(db: &DbConn) -> Result<(), DbErr> {
//! // join paths:
//! // cake -> fruit
//! //      -> filling -> ingredient
//! let super_cake = cake::Entity::load()
//!     .filter_by_id(42)
//!     .with(fruit::Entity) // 1-1 uses join
//!     .with((filling::Entity, ingredient::Entity)) // M-N uses data loader
//!     .one(db)
//!     .await?
//!     .unwrap();
//!
//! super_cake
//!     == cake::ModelEx {
//!         id: 42,
//!         name: "Black Forest".into(),
//!         fruit: Some(fruit::ModelEx {
//! #           id: 1,
//!             name: "Cherry".into(),
//! #           cake_id: Some(1),
//!         }.into()),
//!         fillings: vec![filling::ModelEx {
//! #           id: 2,
//!             name: "Chocolate".into(),
//! #           vendor_id: None,
//! #           ignored_attr: 0,
//!             ingredients: vec![ingredient::ModelEx {
//! #               id: 3,
//!                 name: "Syrup".into(),
//! #               filling_id: Some(2),
//! #               filling: Default::default(),
//!             }],
//!         }],
//!     };
//! # Ok(())
//! # }
//! ```
//! ### Select
//! SeaORM models 1-N and M-N relationships at the Entity level,
//! letting you traverse many-to-many links through a junction table in a single call.
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
//! let fruit: Option<fruit::Model> = cheese.find_related(Fruit).one(db).await?;
//!
//! // find related models (eager): for 1-1 relations
//! let cake_with_fruit: Vec<(cake::Model, Option<fruit::Model>)> =
//!     Cake::find().find_also_related(Fruit).all(db).await?;
//!
//! // find related models (eager): works for both 1-N and M-N relations
//! let cake_with_fillings: Vec<(cake::Model, Vec<filling::Model>)> = Cake::find()
//!     .find_with_related(Filling) // for M-N relations, two joins are performed
//!     .all(db) // rows are automatically consolidated by left entity
//!     .await?;
//! # Ok(())
//! # }
//! ```
//! ### Nested Select
//!
//! Partial models prevent overfetching by letting you querying only the fields
//! you need; it also makes writing deeply nested relational queries simple.
//! ```
//! # use sea_orm::{DbConn, error::*, entity::*, query::*, tests_cfg::*};
//! # async fn function(db: &DbConn) -> Result<(), DbErr> {
//! use sea_orm::DerivePartialModel;
//!
//! #[derive(DerivePartialModel)]
//! #[sea_orm(entity = "cake::Entity")]
//! struct CakeWithFruit {
//!     id: i32,
//!     name: String,
//!     #[sea_orm(nested)]
//!     fruit: Option<fruit::Model>, // this can be a regular or another partial model
//! }
//!
//! let cakes: Vec<CakeWithFruit> = Cake::find()
//!     .left_join(fruit::Entity) // no need to specify join condition
//!     .into_partial_model() // only the columns in the partial model will be selected
//!     .all(db)
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Insert
//! SeaORM's ActiveModel lets you work directly with Rust data structures and
//! persist them through a simple API.
//! It's easy to insert large batches of rows from different data sources.
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
//! // insert one: Active Record style
//! let apple = apple.insert(db).await?;
//! apple.id == 1;
//! # let apple = fruit::ActiveModel {
//! #     name: Set("Apple".to_owned()),
//! #     ..Default::default() // no need to set primary key
//! # };
//!
//! // insert one: repository style
//! let result = Fruit::insert(apple).exec(db).await?;
//! result.last_insert_id == 1;
//! # let apple = fruit::ActiveModel {
//! #     name: Set("Apple".to_owned()),
//! #     ..Default::default() // no need to set primary key
//! # };
//!
//! // insert many returning last insert id
//! let result = Fruit::insert_many([apple, pear]).exec(db).await?;
//! result.last_insert_id == Some(2);
//! # Ok(())
//! # }
//! ```
//!
//! ### Insert (advanced)
//! You can take advantage of database specific features to perform upsert and idempotent insert.
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
//!     .exec_with_returning(db)
//!     .await?;
//! models[0]
//!     == fruit::Model {
//!         id: 1, // database assigned value
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
//!
//! ### Update
//! ActiveModel avoids race conditions by updating only the fields you've changed,
//! never overwriting untouched columns.
//! You can also craft complex bulk update queries with a fluent query building API.
//! ```
//! # use sea_orm::{DbConn, error::*, entity::*, query::*, tests_cfg::*};
//! use fruit::Column::CakeId;
//! use sea_orm::sea_query::{Expr, Value};
//!
//! # async fn function(db: &DbConn) -> Result<(), DbErr> {
//! let pear: Option<fruit::Model> = Fruit::find_by_id(1).one(db).await?;
//! let mut pear: fruit::ActiveModel = pear.unwrap().into();
//!
//! pear.name = Set("Sweet pear".to_owned()); // update value of a single field
//!
//! // update one: only changed columns will be updated
//! let pear: fruit::Model = pear.update(db).await?;
//!
//! // update many: UPDATE "fruit" SET "cake_id" = "cake_id" + 2
//! //               WHERE "fruit"."name" LIKE '%Apple%'
//! Fruit::update_many()
//!     .col_expr(CakeId, Expr::col(CakeId).add(Expr::val(2)))
//!     .filter(fruit::Column::Name.contains("Apple"))
//!     .exec(db)
//!     .await?;
//! # Ok(())
//! # }
//! ```
//! ### Save
//! You can perform "insert or update" operation with ActiveModel, making it easy to compose transactional operations.
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
//! banana.id == Unchanged(2);
//! banana.name = Set("Banana Mongo".to_owned());
//!
//! // update, because primary key `id` is present
//! let banana = banana.save(db).await?;
//! # Ok(())
//! # }
//! ```
//! ### Delete
//! The same ActiveModel API consistent with insert and update.
//! ```
//! # use sea_orm::{DbConn, error::*, entity::*, query::*, tests_cfg::*};
//! # async fn function(db: &DbConn) -> Result<(), DbErr> {
//! // delete one: Active Record style
//! let orange: Option<fruit::Model> = Fruit::find_by_id(1).one(db).await?;
//! let orange: fruit::Model = orange.unwrap();
//! orange.delete(db).await?;
//!
//! // delete one: repository style
//! let orange = fruit::ActiveModel {
//!     id: Set(2),
//!     ..Default::default()
//! };
//! fruit::Entity::delete(orange).exec(db).await?;
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
//! ### Ergonomic Raw SQL
//! Let SeaORM handle 90% of all the transactional queries.
//! When your query is too complex to express, SeaORM still offer convenience in writing raw SQL.
//!
//! The `raw_sql!` macro is like the `format!` macro but without the risk of SQL injection.
//! It supports nested parameter interpolation, array and tuple expansion, and even repeating group,
//! offering great flexibility in crafting complex queries.
//!
//! ```
//! # use sea_orm::{DbErr, DbConn};
//! # async fn function(db: &DbConn) -> Result<(), DbErr> {
//! # use sea_orm::{entity::*, query::*, tests_cfg::*, raw_sql};
//! # struct Item { id: i32 }
//! let item = Item { id: 2 }; // nested parameter access
//!
//! let cake: Option<cake::Model> = Cake::find()
//!     .from_raw_sql(raw_sql!(
//!         Sqlite,
//!         r#"SELECT "id", "name" FROM "cake" WHERE id = {item.id}"#
//!     ))
//!     .one(db)
//!     .await?;
//! # Ok(())
//! # }
//! ```
//! ```
//! # use sea_orm::{DbErr, DbConn};
//! # async fn functio(db: &DbConn) -> Result<(), DbErr> {
//! # use sea_orm::{query::*, FromQueryResult, raw_sql};
//! #[derive(FromQueryResult)]
//! struct CakeWithBakery {
//!     name: String,
//!     #[sea_orm(nested)]
//!     bakery: Option<Bakery>,
//! }
//!
//! #[derive(FromQueryResult)]
//! struct Bakery {
//!     #[sea_orm(alias = "bakery_name")]
//!     name: String,
//! }
//!
//! let cake_ids = [2, 3, 4]; // expanded by the `..` operator
//!
//! // can use many APIs with raw SQL, including nested select
//! let cake: Option<CakeWithBakery> = CakeWithBakery::find_by_statement(raw_sql!(
//!     Sqlite,
//!     r#"SELECT "cake"."name", "bakery"."name" AS "bakery_name"
//!        FROM "cake"
//!        LEFT JOIN "bakery" ON "cake"."bakery_id" = "bakery"."id"
//!        WHERE "cake"."id" IN ({..cake_ids})"#
//! ))
//! .one(db)
//! .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## 🧭 Seaography: instant GraphQL API
//!
//! [Seaography](https://github.com/SeaQL/seaography) is a GraphQL framework built for SeaORM.
//! Seaography allows you to build GraphQL resolvers quickly.
//! With just a few commands, you can launch a fullly-featured GraphQL server from SeaORM entities,
//! complete with filter, pagination, relational queries and mutations!
//!
//! Look at the [Seaography Example](https://github.com/SeaQL/sea-orm/tree/master/examples/seaography_example) to learn more.
//!
//! <img src="https://raw.githubusercontent.com/SeaQL/sea-orm/master/examples/seaography_example/Seaography%20example.png"/>
//!
//! ## 🖥️ SeaORM Pro: Professional Admin Panel
//!
//! [SeaORM Pro](https://github.com/SeaQL/sea-orm-pro/) is an admin panel solution allowing you to quickly and easily launch an admin panel for your application - frontend development skills not required, but certainly nice to have!
//!
//! SeaORM Pro will be updated to support the latest features in SeaORM 2.0.
//!
//! Features:
//!
//! + Full CRUD
//! + Built on React + GraphQL
//! + Built-in GraphQL resolver
//! + Customize the UI with TOML config
//! + Custom GraphQL endpoints *(new in 2.0)*
//! + Role Based Access Control *(new in 2.0)*
//!
//! Learn More
//!
//! + [Example Repo](https://github.com/SeaQL/sea-orm-pro)
//! + [Getting Started](https://www.sea-ql.org/sea-orm-pro/docs/install-and-config/getting-started/)
//!
//! ![](https://raw.githubusercontent.com/SeaQL/sea-orm/refs/heads/master/docs/sea-orm-pro-dark.png#gh-dark-mode-only)
//! ![](https://raw.githubusercontent.com/SeaQL/sea-orm/refs/heads/master/docs/sea-orm-pro-light.png#gh-light-mode-only)
//!
//! ## SQL Server Support
//!
//! [SQL Server for SeaORM](https://www.sea-ql.org/SeaORM-X/) offers the same SeaORM API for MSSQL. We ported all test cases and examples, complemented by MSSQL specific documentation. If you are building enterprise software, you can [request commercial access](https://forms.office.com/r/1MuRPJmYBR). It is currently based on SeaORM 1.0, but we will offer free upgrade to existing users when SeaORM 2.0 is finalized.
//!
//! ## Releases
//!
//! SeaORM 2.0 has reached its release candidate phase. We'd love for you to try it out and help shape the final release by [sharing your feedback](https://github.com/SeaQL/sea-orm/discussions/2548).
//!
//! SeaORM 2.0 is shaping up to be our most significant release yet - with a few breaking changes, plenty of enhancements, and a clear focus on developer experience.
//!
//! + [A Sneak Peek at SeaORM 2.0](https://www.sea-ql.org/blog/2025-09-16-sea-orm-2.0/)
//! + [SeaORM 2.0: A closer look](https://www.sea-ql.org/blog/2025-09-24-sea-orm-2.0/)
//! + [Role Based Access Control in SeaORM 2.0](https://www.sea-ql.org/blog/2025-09-30-sea-orm-rbac/)
//!
//! If you make extensive use of SeaORM's underlying query builder, we recommend checking out our blog post on SeaQuery 1.0 release:
//!
//! + [The road to SeaQuery 1.0](https://www.sea-ql.org/blog/2025-08-30-sea-query-1.0/)
//!
//! It doesn't mean that SeaORM is 'done', we've designed an architecture to allow us to deliver new features without major breaking changes.
//!
//! + [Change Log](https://github.com/SeaQL/sea-orm/tree/master/CHANGELOG.md)
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
//! ## Who's using SeaORM?
//!
//! SeaORM is trusted by companies and startups for both internal tools and public‑facing applications, thanks to its ergonomics and the familiarity it brings from dynamic languages.
//! Built on async Rust, it combines high performance and a strong type system without sacrificing developer productivity.
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
//! We're grateful to our silver sponsors: Digital Ocean, for sponsoring our servers. And JetBrains, for sponsoring our IDE.
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
pub mod dynamic;
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
#[cfg(feature = "rbac")]
#[cfg_attr(docsrs, doc(cfg(feature = "rbac")))]
pub mod rbac;
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
    DeriveActiveEnum, DeriveActiveModel, DeriveActiveModelBehavior, DeriveColumn, DeriveDisplay,
    DeriveEntity, DeriveEntityModel, DeriveIden, DeriveIntoActiveModel, DeriveMigrationName,
    DeriveModel, DeriveModelEx, DerivePartialModel, DerivePrimaryKey, DeriveRelatedEntity,
    DeriveRelation, DeriveValueType, FromJsonQueryResult, FromQueryResult, raw_sql,
    sea_orm_model as model,
};

pub use sea_query;
pub use sea_query::Iden;

pub use sea_orm_macros::EnumIter;
pub use strum;

#[cfg(feature = "sqlx-dep")]
pub use sqlx;
