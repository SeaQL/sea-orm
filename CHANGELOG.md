# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## 0.3.0 - 2021-10-15

https://www.sea-ql.org/SeaORM/blog/2021-10-15-whats-new-in-0.3.0

- Built-in Rocket support
- `ConnectOptions`

```rust
let mut opt = ConnectOptions::new("protocol://username:password@host/database".to_owned());
opt.max_connections(100)
    .min_connections(5)
    .connect_timeout(Duration::from_secs(8))
    .idle_timeout(Duration::from_secs(8));
let db = Database::connect(opt).await?;
```

- [[#211]] Throw error if none of the db rows are affected

```rust
assert_eq!(
    Update::one(cake::ActiveModel {
        name: Set("Cheese Cake".to_owned()),
        ..model.into_active_model()
    })
    .exec(&db)
    .await,
    Err(DbErr::RecordNotFound(
        "None of the database rows are affected".to_owned()
    ))
);

assert_eq!(
    Update::many(cake::Entity)
        .col_expr(cake::Column::Name, Expr::value("Cheese Cake".to_owned()))
        .filter(cake::Column::Id.eq(2))
        .exec(&db)
        .await,
    Ok(UpdateResult { rows_affected: 0 })
);
```

- [[#223]] `ActiveValue::take()` & `ActiveValue::into_value()` without `unwrap()`
- [[#205]] Drop `Default` trait bound of `PrimaryKeyTrait::ValueType`
- [[#222]] Transaction & streaming
- [[#210]] Update `ActiveModelBehavior` API
- [[#240]] Add derive `DeriveIntoActiveModel` and `IntoActiveValue` trait
- [[#237]] Introduce optional serde support for model code generation
- [[#246]] Add `#[automatically_derived]` to all derived implementations

[#211]: https://github.com/SeaQL/sea-orm/pull/211
[#223]: https://github.com/SeaQL/sea-orm/pull/223
[#205]: https://github.com/SeaQL/sea-orm/pull/205
[#222]: https://github.com/SeaQL/sea-orm/pull/222
[#210]: https://github.com/SeaQL/sea-orm/pull/210
[#240]: https://github.com/SeaQL/sea-orm/pull/240
[#237]: https://github.com/SeaQL/sea-orm/pull/237
[#246]: https://github.com/SeaQL/sea-orm/pull/246

## 0.2.6 - 2021-10-09

- [[#224]] [sea-orm-cli] Date & Time column type mapping
- Escape rust keywords with `r#` raw identifier

[#224]: https://github.com/SeaQL/sea-orm/pull/224

## 0.2.5 - 2021-10-06

- [[#227]] Resolve "Inserting actual none value of Option<Date> results in panic"
- [[#219]] [sea-orm-cli] Add `--tables` option
- [[#189]] Add `debug_query` and `debug_query_stmt` macro

[#227]: https://github.com/SeaQL/sea-orm/issues/227
[#219]: https://github.com/SeaQL/sea-orm/pull/219
[#189]: https://github.com/SeaQL/sea-orm/pull/189

## 0.2.4 - 2021-10-01

https://www.sea-ql.org/SeaORM/blog/2021-10-01-whats-new-in-0.2.4

- [[#186]] [sea-orm-cli] Foreign key handling
- [[#191]] [sea-orm-cli] Unique key handling
- [[#182]] `find_linked` join with alias
- [[#202]] Accept both `postgres://` and `postgresql://`
- [[#208]] Support feteching T, (T, U), (T, U, P) etc
- [[#209]] Rename column name & column enum variant
- [[#207]] Support `chrono::NaiveDate` & `chrono::NaiveTime`
- Support `Condition::not` (from sea-query)

[#186]: https://github.com/SeaQL/sea-orm/issues/186
[#191]: https://github.com/SeaQL/sea-orm/issues/191
[#182]: https://github.com/SeaQL/sea-orm/pull/182
[#202]: https://github.com/SeaQL/sea-orm/pull/202
[#208]: https://github.com/SeaQL/sea-orm/pull/208
[#209]: https://github.com/SeaQL/sea-orm/pull/209
[#207]: https://github.com/SeaQL/sea-orm/pull/207

## 0.2.3 - 2021-09-22

- [[#152]] DatabaseConnection impl `Clone`
- [[#175]] Impl `TryGetableMany` for diffrent types of generics
- Codegen `TimestampWithTimeZone` fixup

[#152]: https://github.com/SeaQL/sea-orm/issues/152
[#175]: https://github.com/SeaQL/sea-orm/issues/175

## 0.2.2 - 2021-09-18

- [[#105]] Compact entity format
- [[#132]] Add ActiveModel `insert` & `update`
- [[#129]] Add `set` method to `UpdateMany`
- [[#118]] Initial lock support
- [[#167]] Add `FromQueryResult::find_by_statement`

[#105]: https://github.com/SeaQL/sea-orm/issues/105
[#132]: https://github.com/SeaQL/sea-orm/issues/132
[#129]: https://github.com/SeaQL/sea-orm/issues/129
[#118]: https://github.com/SeaQL/sea-orm/issues/118
[#167]: https://github.com/SeaQL/sea-orm/issues/167

## 0.2.1 - 2021-09-04

- Update dependencies

## 0.2.0 - 2021-09-03

- [[#37]] Rocket example
- [[#114]] `log` crate and `env-logger`
- [[#103]] `InsertResult` to return the primary key's type
- [[#89]] Represent several relations between same types by `Linked`
- [[#59]] Transforming an Entity into `TableCreateStatement`

[#37]: https://github.com/SeaQL/sea-orm/issues/37
[#114]: https://github.com/SeaQL/sea-orm/issues/114
[#103]: https://github.com/SeaQL/sea-orm/issues/103
[#89]: https://github.com/SeaQL/sea-orm/issues/89
[#59]: https://github.com/SeaQL/sea-orm/issues/59

## 0.1.3 - 2021-08-30

- [[#108]] Remove impl TryGetable for Option<T>

[#108]: https://github.com/SeaQL/sea-orm/issues/108

## 0.1.2 - 2021-08-23

- [[#68]] Added `DateTimeWithTimeZone` as supported attribute type
- [[#70]] Generate arbitrary named entity
- [[#80]] Custom column name
- [[#81]] Support join on multiple columns
- [[#99]] Implement FromStr for ColumnTrait

[#68]: https://github.com/SeaQL/sea-orm/issues/68
[#70]: https://github.com/SeaQL/sea-orm/issues/70
[#80]: https://github.com/SeaQL/sea-orm/issues/80
[#81]: https://github.com/SeaQL/sea-orm/issues/81
[#99]: https://github.com/SeaQL/sea-orm/issues/99

## 0.1.1 - 2021-08-08

- Early release of SeaORM
