<div align="center">

  <img src="https://www.sea-ql.org/SeaORM/img/SeaORM banner.png"/>

  <h1>SeaORM</h1>

  <h3>🐚 An async & dynamic ORM for Rust</h3>

  [![crate](https://img.shields.io/crates/v/sea-orm.svg)](https://crates.io/crates/sea-orm)
  [![docs](https://docs.rs/sea-orm/badge.svg)](https://docs.rs/sea-orm)
  [![build status](https://github.com/SeaQL/sea-orm/actions/workflows/rust.yml/badge.svg)](https://github.com/SeaQL/sea-orm/actions/workflows/rust.yml)

</div>

# SeaORM

#### SeaORM is a relational ORM to help you build web services in Rust with the familiarity of dynamic languages.

[![GitHub stars](https://img.shields.io/github/stars/SeaQL/sea-orm.svg?style=social&label=Star&maxAge=1)](https://github.com/SeaQL/sea-orm/stargazers/)
If you like what we do, consider starring, sharing and contributing!

Please help us with maintaining SeaORM by completing the [SeaQL Community Survey 2024](https://sea-ql.org/community-survey)!

[![Discord](https://img.shields.io/discord/873880840487206962?label=Discord)](https://discord.com/invite/uCPdDXzbdv)
Join our Discord server to chat with other members of the SeaQL community!

## Getting Started

+ [Documentation](https://www.sea-ql.org/SeaORM)
+ [Tutorial](https://www.sea-ql.org/sea-orm-tutorial)
+ [Cookbook](https://www.sea-ql.org/sea-orm-cookbook)

Integration examples:

+ [Actix v4 Example](https://github.com/SeaQL/sea-orm/tree/master/examples/actix_example)
+ [Axum Example](https://github.com/SeaQL/sea-orm/tree/master/examples/axum_example)
+ [GraphQL Example](https://github.com/SeaQL/sea-orm/tree/master/examples/graphql_example)
+ [jsonrpsee Example](https://github.com/SeaQL/sea-orm/tree/master/examples/jsonrpsee_example)
+ [Loco TODO Example](https://github.com/SeaQL/sea-orm/tree/master/examples/loco_example) / [Loco REST Starter](https://github.com/SeaQL/sea-orm/tree/master/examples/loco_starter)
+ [Poem Example](https://github.com/SeaQL/sea-orm/tree/master/examples/poem_example)
+ [Rocket Example](https://github.com/SeaQL/sea-orm/tree/master/examples/rocket_example) / [Rocket OpenAPI Example](https://github.com/SeaQL/sea-orm/tree/master/examples/rocket_okapi_example)
+ [Salvo Example](https://github.com/SeaQL/sea-orm/tree/master/examples/salvo_example)
+ [Tonic Example](https://github.com/SeaQL/sea-orm/tree/master/examples/tonic_example)
+ [Seaography Example](https://github.com/SeaQL/sea-orm/tree/master/examples/seaography_example)

## Features

1. Async

    Relying on [SQLx](https://github.com/launchbadge/sqlx), SeaORM is a new library with async support from day 1.

2. Dynamic

    Built upon [SeaQuery](https://github.com/SeaQL/sea-query), SeaORM allows you to build complex dynamic queries.

3. Testable

    Use mock connections and/or SQLite to write tests for your application logic.

4. Service Oriented

    Quickly build services that join, filter, sort and paginate data in REST, GraphQL and gRPC APIs.

## A quick taste of SeaORM

### Entity
```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "cake")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::fruit::Entity")]
    Fruit,
}

impl Related<super::fruit::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Fruit.def()
    }
}
```

### Select
```rust
// find all models
let cakes: Vec<cake::Model> = Cake::find().all(db).await?;

// find and filter
let chocolate: Vec<cake::Model> = Cake::find()
    .filter(cake::Column::Name.contains("chocolate"))
    .all(db)
    .await?;

// find one model
let cheese: Option<cake::Model> = Cake::find_by_id(1).one(db).await?;
let cheese: cake::Model = cheese.unwrap();

// find related models (lazy)
let fruits: Vec<fruit::Model> = cheese.find_related(Fruit).all(db).await?;

// find related models (eager)
let cake_with_fruits: Vec<(cake::Model, Vec<fruit::Model>)> =
    Cake::find().find_with_related(Fruit).all(db).await?;

```
### Insert
```rust
let apple = fruit::ActiveModel {
    name: Set("Apple".to_owned()),
    ..Default::default() // no need to set primary key
};

let pear = fruit::ActiveModel {
    name: Set("Pear".to_owned()),
    ..Default::default()
};

// insert one
let pear = pear.insert(db).await?;

// insert many
Fruit::insert_many([apple, pear]).exec(db).await?;
```
### Update
```rust
use sea_orm::sea_query::{Expr, Value};

let pear: Option<fruit::Model> = Fruit::find_by_id(1).one(db).await?;
let mut pear: fruit::ActiveModel = pear.unwrap().into();

pear.name = Set("Sweet pear".to_owned());

// update one
let pear: fruit::Model = pear.update(db).await?;

// update many: UPDATE "fruit" SET "cake_id" = NULL WHERE "fruit"."name" LIKE '%Apple%'
Fruit::update_many()
    .col_expr(fruit::Column::CakeId, Expr::value(Value::Int(None)))
    .filter(fruit::Column::Name.contains("Apple"))
    .exec(db)
    .await?;

```
### Save
```rust
let banana = fruit::ActiveModel {
    id: NotSet,
    name: Set("Banana".to_owned()),
    ..Default::default()
};

// create, because primary key `id` is `NotSet`
let mut banana = banana.save(db).await?;

banana.name = Set("Banana Mongo".to_owned());

// update, because primary key `id` is `Set`
let banana = banana.save(db).await?;

```
### Delete
```rust
// delete one
let orange: Option<fruit::Model> = Fruit::find_by_id(1).one(db).await?;
let orange: fruit::Model = orange.unwrap();
fruit::Entity::delete(orange.into_active_model())
    .exec(db)
    .await?;

// or simply
let orange: Option<fruit::Model> = Fruit::find_by_id(1).one(db).await?;
let orange: fruit::Model = orange.unwrap();
orange.delete(db).await?;

// delete many: DELETE FROM "fruit" WHERE "fruit"."name" LIKE 'Orange'
fruit::Entity::delete_many()
    .filter(fruit::Column::Name.contains("Orange"))
    .exec(db)
    .await?;

```

## 🧭 Seaography: GraphQL integration (preview)

[Seaography](https://github.com/SeaQL/seaography) is a GraphQL framework built on top of SeaORM. Seaography allows you to build GraphQL resolvers quickly. With just a few commands, you can launch a GraphQL server from SeaORM entities!

Starting `0.12`, `seaography` integration is built into `sea-orm`. While Seaography development is still in an early stage, it is especially useful in prototyping and building internal-use admin panels.

<img src="https://raw.githubusercontent.com/SeaQL/sea-orm/master/examples/seaography_example/Seaography%20example.png"/>

Look at the [Seaography Example](https://github.com/SeaQL/sea-orm/tree/master/examples/seaography_example) to learn more.

## Learn More

1. [Design](https://github.com/SeaQL/sea-orm/tree/master/DESIGN.md)
1. [Architecture](https://www.sea-ql.org/SeaORM/docs/internal-design/architecture/)
1. [Engineering](https://www.sea-ql.org/blog/2022-07-30-engineering/)
1. [Change Log](https://github.com/SeaQL/sea-orm/tree/master/CHANGELOG.md)

### Who's using SeaORM?

See [Built with SeaORM](https://github.com/SeaQL/sea-orm/blob/master/COMMUNITY.md#built-with-seaorm). Feel free to [submit yours](https://github.com/SeaQL/sea-orm/issues/403)!

## License

Licensed under either of

-   Apache License, Version 2.0
    ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
-   MIT license
    ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

SeaORM is a community driven project. We welcome you to participate, contribute and together help build Rust's future.

A big shout out to our contributors!

[![Contributors](https://opencollective.com/sea-orm/contributors.svg?width=1000&button=false)](https://github.com/SeaQL/sea-orm/graphs/contributors)

## Sponsorship

[SeaQL.org](https://www.sea-ql.org/) is an independent open-source organization run by passionate developers. If you enjoy using our libraries, please star and share our repositories. If you feel generous, a small donation via [GitHub Sponsor](https://github.com/sponsors/SeaQL) will be greatly appreciated, and goes a long way towards sustaining the organization.

We invite you to participate, contribute and together help build Rust's future.

### Gold Sponsors

<a href="https://osmos.io/">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://www.sea-ql.org/static/sponsors/Osmos-dark.svg">
    <img src="https://www.sea-ql.org/static/sponsors/Osmos.svg" width="238">
  </picture>
</a>

## Mascot

A friend of Ferris, Terres the hermit crab is the official mascot of SeaORM. His hobby is collecting shells.

<img alt="Terres" src="https://www.sea-ql.org/SeaORM/img/Terres.png" width="400"/>
