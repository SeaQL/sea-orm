# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## 0.8.0 - Pending

### New Features
* [sea-orm-cli] `sea migrate generate` to generate a new, empty migration file https://github.com/SeaQL/sea-orm/pull/656

### Enhancements
* Add `max_connections` option to CLI https://github.com/SeaQL/sea-orm/pull/670
* Derive `Eq`, `Clone` for `DbErr` https://github.com/SeaQL/sea-orm/pull/677
* Add `is_changed` to `ActiveModelTrait` https://github.com/SeaQL/sea-orm/pull/683

### Bug Fixes
* Fix `DerivePrimaryKey` with custom primary key column name https://github.com/SeaQL/sea-orm/pull/694
* Fix `DeriveEntityModel` macros override column name https://github.com/SeaQL/sea-orm/pull/695

### Breaking changes
* Migration utilities are moved from sea-schema to sea-orm repo, under a new sub-crate `sea-orm-migration`. `sea_schema::migration::prelude` should be replaced by `sea_orm_migration::prelude` in all migration files

### Upgrades
* Upgrade `sea-query` to 0.24.x, `sea-schema` to 0.8.x
* Upgrade example to Actix Web 4, Actix Web 3 remains https://github.com/SeaQL/sea-orm/pull/638
* Added Tonic gRPC example https://github.com/SeaQL/sea-orm/pull/659
* Upgrade GraphQL example to use axum 0.5.x
* Upgrade axum example to 0.5.x

## 0.7.1 - 2022-03-26

* Fix sea-orm-cli error
* Fix sea-orm cannot build without `with-json`

## 0.7.0 - 2022-03-26

### New Features
* Update ActiveModel by JSON by @billy1624 in https://github.com/SeaQL/sea-orm/pull/492
* Supports `time` crate by @billy1624 https://github.com/SeaQL/sea-orm/pull/602
* Allow for creation of indexes for PostgreSQL and SQLite @nickb937 https://github.com/SeaQL/sea-orm/pull/593
* Added `delete_by_id` @ShouvikGhosh2048 https://github.com/SeaQL/sea-orm/pull/590
* Implement `PaginatorTrait` for `SelectorRaw` @shinbunbun https://github.com/SeaQL/sea-orm/pull/617

### Enhancements
* Added axum graphql example by @aaronleopold in https://github.com/SeaQL/sea-orm/pull/587
* Add example for integrate with jsonrpsee by @hunjixin https://github.com/SeaQL/sea-orm/pull/632
* Codegen add serde derives to enums, if specified by @BenJeau https://github.com/SeaQL/sea-orm/pull/463
* Codegen Unsigned Integer by @billy1624 https://github.com/SeaQL/sea-orm/pull/397
* Add `Send` bound to `QueryStream` and `TransactionStream` by @sebpuetz https://github.com/SeaQL/sea-orm/pull/471
* Add `Send` to `StreamTrait` by @nappa85 https://github.com/SeaQL/sea-orm/pull/622
* `sea` as an alternative bin name to `sea-orm-cli` by @ZhangHanDong https://github.com/SeaQL/sea-orm/pull/558

### Bug Fixes
* Fix codegen with Enum in expanded format by @billy1624 https://github.com/SeaQL/sea-orm/pull/624
* Fixing and testing into_json of various field types by @billy1624 https://github.com/SeaQL/sea-orm/pull/539

### Breaking changes
* Exclude `mock` from default features by @billy1624 https://github.com/SeaQL/sea-orm/pull/562
* `create_table_from_entity` will no longer create index for MySQL, please use the new method `create_index_from_entity`

### Documentations
* Describe default value of ActiveValue on document by @Ken-Miura in https://github.com/SeaQL/sea-orm/pull/556
* community: add axum-book-management by @lz1998 in https://github.com/SeaQL/sea-orm/pull/564
* Add Backpack to project showcase by @JSH32 in https://github.com/SeaQL/sea-orm/pull/567
* Add mediarepo to showcase by @Trivernis in https://github.com/SeaQL/sea-orm/pull/569
* COMMUNITY: add a link to Svix to showcase by @tasn in https://github.com/SeaQL/sea-orm/pull/537
* Update COMMUNITY.md by @naryand in https://github.com/SeaQL/sea-orm/pull/570
* Update COMMUNITY.md by @BobAnkh in https://github.com/SeaQL/sea-orm/pull/568
* Update COMMUNITY.md by @KaniyaSimeji in https://github.com/SeaQL/sea-orm/pull/566
* Update COMMUNITY.md by @aaronleopold in https://github.com/SeaQL/sea-orm/pull/565
* Update COMMUNITY.md by @gudaoxuri in https://github.com/SeaQL/sea-orm/pull/572
* Update Wikijump's entry in COMMUNITY.md by @ammongit in https://github.com/SeaQL/sea-orm/pull/573
* Update COMMUNITY.md by @koopa1338 in https://github.com/SeaQL/sea-orm/pull/574
* Update COMMUNITY.md by @gengteng in https://github.com/SeaQL/sea-orm/pull/580
* Update COMMUNITY.md by @Yama-Tomo in https://github.com/SeaQL/sea-orm/pull/582
* add oura-postgres-sink to COMMUNITY.md by @rvcas in https://github.com/SeaQL/sea-orm/pull/594
* Add rust-example-caster-api to COMMUNITY.md by @bkonkle in https://github.com/SeaQL/sea-orm/pull/623

### Fixed Issues
* orm-cli generated incorrect type for #[sea_orm(primary_key)]. Should be u64. Was i64. https://github.com/SeaQL/sea-orm/issues/295
* how to update dynamicly from json value https://github.com/SeaQL/sea-orm/issues/346
* Make `DatabaseConnection` `Clone` with the default features enabled https://github.com/SeaQL/sea-orm/issues/438
* Updating mutiple fields in a Model by passing a reference https://github.com/SeaQL/sea-orm/issues/460
* SeaORM CLI not adding serde derives to Enums https://github.com/SeaQL/sea-orm/issues/461
* sea-orm-cli generates wrong datatype for nullable blob https://github.com/SeaQL/sea-orm/issues/490
* Support the time crate in addition (instead of?) chrono https://github.com/SeaQL/sea-orm/issues/499
* PaginatorTrait for SelectorRaw https://github.com/SeaQL/sea-orm/issues/500
* sea_orm::DatabaseConnection should implement `Clone` by default https://github.com/SeaQL/sea-orm/issues/517
* How do you seed data in migrations using ActiveModels? https://github.com/SeaQL/sea-orm/issues/522
* Datetime fields are not serialized by `.into_json()` on queries https://github.com/SeaQL/sea-orm/issues/530
* Update / Delete by id https://github.com/SeaQL/sea-orm/issues/552
* `#[sea_orm(indexed)]` only works for MySQL https://github.com/SeaQL/sea-orm/issues/554
* `sea-orm-cli generate --with-serde` does not work on Postegresql custom type https://github.com/SeaQL/sea-orm/issues/581
* `sea-orm-cli generate --expanded-format` panic when postgres table contains enum type https://github.com/SeaQL/sea-orm/issues/614
* UUID fields are not serialized by `.into_json()` on queries https://github.com/SeaQL/sea-orm/issues/619

**Full Changelog**: https://github.com/SeaQL/sea-orm/compare/0.6.0...0.7.0

## 0.6.0 - 2022-02-07

### New Features
* Migration Support by @billy1624 in https://github.com/SeaQL/sea-orm/pull/335
* Support `DateTime<Utc>` & `DateTime<Local>` by @billy1624 in https://github.com/SeaQL/sea-orm/pull/489
* Add `max_lifetime` connection option by @billy1624 in https://github.com/SeaQL/sea-orm/pull/493

### Enhancements
* Model with Generics by @billy1624 in https://github.com/SeaQL/sea-orm/pull/400
* Add Poem example by @sunli829 in https://github.com/SeaQL/sea-orm/pull/446
* Codegen `column_name` proc_macro attribute by @billy1624 in https://github.com/SeaQL/sea-orm/pull/433
* Easy joins with MockDatabase #447 by @cemoktra in https://github.com/SeaQL/sea-orm/pull/455

### Bug Fixes
* CLI allow generate entity with url without password by @billy1624 in https://github.com/SeaQL/sea-orm/pull/436
* Support up to 6-ary composite primary key by @billy1624 in https://github.com/SeaQL/sea-orm/pull/423
* Fix FromQueryResult when Result is redefined by @tasn in https://github.com/SeaQL/sea-orm/pull/495
* Remove `r#` prefix when deriving `FromQueryResult` by @smrtrfszm in https://github.com/SeaQL/sea-orm/pull/494

### Breaking Changes
* Name conflict of foreign key constraints when two entities have more than one foreign keys by @billy1624 in https://github.com/SeaQL/sea-orm/pull/417

### Fixed Issues
* Is it possible to have 4 values Composite Key? https://github.com/SeaQL/sea-orm/issues/352
* Support `DateTime<Utc>` & `DateTime<Local>` https://github.com/SeaQL/sea-orm/issues/381
* Codegen `column_name` proc_macro attribute if column name isn't in snake case https://github.com/SeaQL/sea-orm/issues/395
* Model with Generics https://github.com/SeaQL/sea-orm/issues/402
* Foreign key constraint collision when multiple keys exist between the same two tables https://github.com/SeaQL/sea-orm/issues/405
* sea-orm-cli passwordless database user causes "No password was found in the database url" error https://github.com/SeaQL/sea-orm/issues/435
* Testing joins with MockDatabase https://github.com/SeaQL/sea-orm/issues/447
* Surface max_lifetime connection option https://github.com/SeaQL/sea-orm/issues/475

**Full Changelog**: https://github.com/SeaQL/sea-orm/compare/0.5.0...0.6.0

## 0.5.0 - 2022-01-01

### Fixed Issues
* Why insert, update, etc return a ActiveModel instead of Model? https://github.com/SeaQL/sea-orm/issues/289
* Rework `ActiveValue` https://github.com/SeaQL/sea-orm/issues/321
* Some missing ActiveEnum utilities https://github.com/SeaQL/sea-orm/issues/338

### Merged PRs
* First metric and tracing implementation by @nappa85 in https://github.com/SeaQL/sea-orm/pull/373
* Update sea-orm to depends on SeaQL/sea-query#202 by @billy1624 in https://github.com/SeaQL/sea-orm/pull/370
* Codegen ActiveEnum & Create Enum From ActiveEnum by @billy1624 in https://github.com/SeaQL/sea-orm/pull/348
* Axum example: update to Axum v0.4.2 by @ttys3 in https://github.com/SeaQL/sea-orm/pull/383
* Fix rocket version by @Gabriel-Paulucci in https://github.com/SeaQL/sea-orm/pull/384
* Insert & Update Return `Model` by @billy1624 in https://github.com/SeaQL/sea-orm/pull/339
* Rework `ActiveValue` by @billy1624 in https://github.com/SeaQL/sea-orm/pull/340
* Add wrapper method `ModelTrait::delete` by @billy1624 in https://github.com/SeaQL/sea-orm/pull/396
* Add docker create script for contributors to setup databases locally by @billy1624 in https://github.com/SeaQL/sea-orm/pull/378
* Log with tracing-subscriber by @billy1624 in https://github.com/SeaQL/sea-orm/pull/399
* Codegen SQLite by @billy1624 in https://github.com/SeaQL/sea-orm/pull/386
* PR without clippy warmings in file changed tab by @billy1624 in https://github.com/SeaQL/sea-orm/pull/401
* Rename `sea-strum` lib back to `strum` by @billy1624 in https://github.com/SeaQL/sea-orm/pull/361

### Breaking Changes
* `ActiveModel::insert` and `ActiveModel::update` return `Model` instead of `ActiveModel`
* Method `ActiveModelBehavior::after_save` takes `Model` as input instead of `ActiveModel`
* Rename method `sea_orm::unchanged_active_value_not_intended_for_public_use` to `sea_orm::Unchanged`
* Rename method `ActiveValue::unset` to `ActiveValue::not_set`
* Rename method `ActiveValue::is_unset` to `ActiveValue::is_not_set`
* `PartialEq` of `ActiveValue` will also check the equality of state instead of just checking the equality of value

**Full Changelog**: https://github.com/SeaQL/sea-orm/compare/0.4.2...0.5.0

## 0.4.2 - 2021-12-12

### Fixed Issues
* Delete::many() doesn't work when schema_name is defined https://github.com/SeaQL/sea-orm/issues/362
* find_with_related panic https://github.com/SeaQL/sea-orm/issues/374
* How to define rust type of TIMESTAMP? https://github.com/SeaQL/sea-orm/issues/344
* Add Table on the generated Column enum https://github.com/SeaQL/sea-orm/issues/356

### Merged PRs
* `Delete::many()` with `TableRef` by @billy1624 in https://github.com/SeaQL/sea-orm/pull/363
* Fix related & linked with enum columns by @billy1624 in https://github.com/SeaQL/sea-orm/pull/376
* Temporary Fix: Handling MySQL & SQLite timestamp columns by @billy1624 in https://github.com/SeaQL/sea-orm/pull/379
* Add feature to generate table Iden by @Sytten in https://github.com/SeaQL/sea-orm/pull/360

**Full Changelog**: https://github.com/SeaQL/sea-orm/compare/0.4.1...0.4.2

## 0.4.1 - 2021-12-05

### Fixed Issues
* Is it possible to have 4 values Composite Key? https://github.com/SeaQL/sea-orm/issues/352
* [sea-orm-cli] Better handling of relation generations https://github.com/SeaQL/sea-orm/issues/239

### Merged PRs
* Add TryFromU64 trait for `DateTime<FixedOffset>`. by @kev0960 in https://github.com/SeaQL/sea-orm/pull/331
* add offset and limit by @lz1998 in https://github.com/SeaQL/sea-orm/pull/351
* For some reason the `axum_example` fail to compile by @billy1624 in https://github.com/SeaQL/sea-orm/pull/355
* Support Up to 6 Values Composite Primary Key by @billy1624 in https://github.com/SeaQL/sea-orm/pull/353
* Codegen Handle Self Referencing & Multiple Relations to the Same Related Entity by @billy1624 in https://github.com/SeaQL/sea-orm/pull/347

**Full Changelog**: https://github.com/SeaQL/sea-orm/compare/0.4.0...0.4.1

## 0.4.0 - 2021-11-19

### Fixed Issues
* Disable SQLx query logging https://github.com/SeaQL/sea-orm/issues/290
* Code generated by `sea-orm-cli` cannot pass clippy https://github.com/SeaQL/sea-orm/issues/296
* Should return detailed error message for connection failure https://github.com/SeaQL/sea-orm/issues/310
* `DateTimeWithTimeZone` does not implement `Serialize` and `Deserialize` https://github.com/SeaQL/sea-orm/issues/319
* Support returning clause to avoid database hits https://github.com/SeaQL/sea-orm/issues/183

### Merged PRs
* chore: update to Rust 2021 Edition by @sno2 in https://github.com/SeaQL/sea-orm/pull/273
* Enumeration - 3 by @billy1624 in https://github.com/SeaQL/sea-orm/pull/274
* Enumeration - 2 by @billy1624 in https://github.com/SeaQL/sea-orm/pull/261
* Codegen fix clippy warnings by @billy1624 in https://github.com/SeaQL/sea-orm/pull/303
* Add axum example by @YoshieraHuang in https://github.com/SeaQL/sea-orm/pull/297
* Enumeration by @billy1624 in https://github.com/SeaQL/sea-orm/pull/258
* Add `PaginatorTrait` and `CountTrait` for more constrains by @YoshieraHuang in https://github.com/SeaQL/sea-orm/pull/306
* Continue `PaginatorTrait` by @billy1624 in https://github.com/SeaQL/sea-orm/pull/307
* Refactor `Schema` by @billy1624 in https://github.com/SeaQL/sea-orm/pull/309
* Detailed connection errors by @billy1624 in https://github.com/SeaQL/sea-orm/pull/312
* Suppress `ouroboros` missing docs warnings by @billy1624 in https://github.com/SeaQL/sea-orm/pull/288
* `with-json` feature requires `chrono/serde` by @billy1624 in https://github.com/SeaQL/sea-orm/pull/320
* Pass the argument `entity.table_ref()` instead of just `entity`. by @josh-codes in https://github.com/SeaQL/sea-orm/pull/318
* Unknown types could be a newtypes instead of `ActiveEnum` by @billy1624 in https://github.com/SeaQL/sea-orm/pull/324
* Returning by @billy1624 in https://github.com/SeaQL/sea-orm/pull/292

### Breaking Changes
* Refactor `paginate()` & `count()` utilities into `PaginatorTrait`. You can use the paginator as usual but you might need to import `PaginatorTrait` manually when upgrading from previous version.
    ```rust
    use futures::TryStreamExt;
    use sea_orm::{entity::*, query::*, tests_cfg::cake};

    let mut cake_stream = cake::Entity::find()
        .order_by_asc(cake::Column::Id)
        .paginate(db, 50)
        .into_stream();

    while let Some(cakes) = cake_stream.try_next().await? {
        // Do something on cakes: Vec<cake::Model>
    }
    ```
* The helper struct `Schema` converting `EntityTrait` into different `sea-query` statement now has to be initialized with `DbBackend`.
    ```rust
    use sea_orm::{tests_cfg::*, DbBackend, Schema};
    use sea_orm::sea_query::TableCreateStatement;

    // 0.3.x
    let _: TableCreateStatement = Schema::create_table_from_entity(cake::Entity);

    // 0.4.x
    let schema: Schema = Schema::new(DbBackend::MySql);
    let _: TableCreateStatement = schema.create_table_from_entity(cake::Entity);
    ```
* When performing insert or update operation on `ActiveModel` against PostgreSQL, `RETURNING` clause will be used to perform select in a single SQL statement.
    ```rust
    // For PostgreSQL
    cake::ActiveModel {
        name: Set("Apple Pie".to_owned()),
        ..Default::default()
    }
    .insert(&postgres_db)
    .await?;

    assert_eq!(
        postgres_db.into_transaction_log(),
        vec![Transaction::from_sql_and_values(
            DbBackend::Postgres,
            r#"INSERT INTO "cake" ("name") VALUES ($1) RETURNING "id", "name""#,
            vec!["Apple Pie".into()]
        )]);
    ```
    ```rust
    // For MySQL & SQLite
    cake::ActiveModel {
        name: Set("Apple Pie".to_owned()),
        ..Default::default()
    }
    .insert(&other_db)
    .await?;

    assert_eq!(
        other_db.into_transaction_log(),
        vec![
            Transaction::from_sql_and_values(
                DbBackend::MySql,
                r#"INSERT INTO `cake` (`name`) VALUES (?)"#,
                vec!["Apple Pie".into()]
            ),
            Transaction::from_sql_and_values(
                DbBackend::MySql,
                r#"SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`id` = ? LIMIT ?"#,
                vec![15.into(), 1u64.into()]
            )]);
    ```

**Full Changelog**: https://github.com/SeaQL/sea-orm/compare/0.3.2...0.4.0

## 0.3.2 - 2021-11-03

### Fixed Issues
* Support for BYTEA Postgres primary keys https://github.com/SeaQL/sea-orm/issues/286

### Merged PRs
* Documentation for sea-orm by @charleschege in https://github.com/SeaQL/sea-orm/pull/280
* Support `Vec<u8>` primary key by @billy1624 in https://github.com/SeaQL/sea-orm/pull/287

**Full Changelog**: https://github.com/SeaQL/sea-orm/compare/0.3.1...0.3.2

## 0.3.1 - 2021-10-23

(We are changing our Changelog format from now on)

### Fixed Issues
* Align case trasforms across derive macros https://github.com/SeaQL/sea-orm/issues/262
* Added `is_null` and `is_not_null` to `ColumnTrait` https://github.com/SeaQL/sea-orm/issues/267

(The following is generated by GitHub)

### Merged PRs
* Changed manual url parsing to use Url crate by @AngelOnFira in https://github.com/SeaQL/sea-orm/pull/253
* Test self referencing relation by @billy1624 in https://github.com/SeaQL/sea-orm/pull/256
* Unify case-transform using the same crate by @billy1624 in https://github.com/SeaQL/sea-orm/pull/264
* CI cleaning by @AngelOnFira in https://github.com/SeaQL/sea-orm/pull/263
* CI install sea-orm-cli in debug mode by @billy1624 in https://github.com/SeaQL/sea-orm/pull/265

**Full Changelog**: https://github.com/SeaQL/sea-orm/compare/0.3.0...0.3.1

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

// update many remains the same
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
