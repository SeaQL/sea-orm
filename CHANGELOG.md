# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## 0.12.0 - Pending

+ Yanked    : `0.12.0-rc.1`
+ 2023-05-19: `0.12.0-rc.2`
+ 2023-06-22: `0.12.0-rc.3`
+ 2023-07-08: `0.12.0-rc.4`

### New Features

* Supports for partial select of `Option<T>` model field. A `None` value will be filled when the select result does not contain the `Option<T>` field without throwing an error. https://github.com/SeaQL/sea-orm/pull/1513
```rust
customer::ActiveModel {
    name: Set("Alice".to_owned()),
    notes: Set(Some("Want to communicate with Bob".to_owned())),
    ..Default::default()
}
.save(db)
.await?;

// The `notes` field was intentionally leaved out
let customer = Customer::find()
    .select_only()
    .column(customer::Column::Id)
    .column(customer::Column::Name)
    .one(db)
    .await
    .unwrap();

// The select result does not contain `notes` field.
// Since it's of type `Option<String>`, it'll be `None` and no error will be thrown.
assert_eq!(customers.notes, None);
```
* [sea-orm-cli] the `migrate init` command will create a `.gitignore` file when the migration folder reside in a Git repository https://github.com/SeaQL/sea-orm/pull/1334
* Added `MigratorTrait::migration_table_name()` method to configure the name of migration table https://github.com/SeaQL/sea-orm/pull/1511
```rust
#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220118_000001_create_cake_table::Migration),
            Box::new(m20220118_000002_create_fruit_table::Migration),
        ]
    }

    // Override the name of migration table
    fn migration_table_name() -> sea_orm::DynIden {
        Alias::new("override_migration_table_name").into_iden()
    }
}
```
* Added option to construct chained AND / OR join on condition https://github.com/SeaQL/sea-orm/pull/1433
```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "cake")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_name = "name", enum_name = "Name")]
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // By default, it's
    // `JOIN `fruit` ON `cake`.`id` = `fruit`.`cake_id` AND `fruit`.`name` LIKE '%tropical%'`
    #[sea_orm(
        has_many = "super::fruit::Entity",
        on_condition = r#"super::fruit::Column::Name.like("%tropical%")"#
    )]
    TropicalFruit,
    // Or specify `condition_type = "any"` to override it,
    // `JOIN `fruit` ON `cake`.`id` = `fruit`.`cake_id` OR `fruit`.`name` LIKE '%tropical%'`
    #[sea_orm(
        has_many = "super::fruit::Entity",
        on_condition = r#"super::fruit::Column::Name.like("%tropical%")"#
        condition_type = "any",
    )]
    OrTropicalFruit,
}

impl ActiveModelBehavior for ActiveModel {}
```
You can also override it in custom join.
```rust
assert_eq!(
    cake::Entity::find()
        .column_as(
            Expr::col((Alias::new("cake_filling_alias"), cake_filling::Column::CakeId)),
            "cake_filling_cake_id"
        )
        .join(JoinType::LeftJoin, cake::Relation::OrTropicalFruit.def())
        .join_as_rev(
            JoinType::LeftJoin,
            cake_filling::Relation::Cake
                .def()
                // chained AND / OR join on condition
                .condition_type(ConditionType::Any)
                .on_condition(|left, _right| {
                    Expr::col((left, cake_filling::Column::CakeId))
                        .gt(10)
                        .into_condition()
                }),
            Alias::new("cake_filling_alias")
        )
        .build(DbBackend::MySql)
        .to_string(),
    [
        "SELECT `cake`.`id`, `cake`.`name`, `cake_filling_alias`.`cake_id` AS `cake_filling_cake_id` FROM `cake`",
        "LEFT JOIN `fruit` ON `cake`.`id` = `fruit`.`cake_id` OR `fruit`.`name` LIKE '%tropical%'",
        "LEFT JOIN `cake_filling` AS `cake_filling_alias` ON `cake_filling_alias`.`cake_id` = `cake`.`id` OR `cake_filling_alias`.`cake_id` > 10",
    ]
    .join(" ")
);
```
* Supports entity with composite primary key of length 12 https://github.com/SeaQL/sea-orm/pull/1508
    * Implemented `IntoIdentity` for `Identity` https://github.com/SeaQL/sea-orm/pull/1508
    * `Identity` supports up to identity tuple of `DynIden` with length up to 12 https://github.com/SeaQL/sea-orm/pull/1508
    * Implemented `IntoIdentity` for tuple of `IdenStatic` with length up to 12 https://github.com/SeaQL/sea-orm/pull/1508
    * Implemented `IdentityOf` for tuple of `ColumnTrait` with length up to 12 https://github.com/SeaQL/sea-orm/pull/1508
    * Implemented `TryGetableMany` for tuple of `TryGetable` with length up to 12 https://github.com/SeaQL/sea-orm/pull/1508
    * Implemented `TryFromU64` for tuple of `TryFromU64` with length up to 12 https://github.com/SeaQL/sea-orm/pull/1508
```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "primary_key_of_12")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id_1: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub id_2: i8,
    #[sea_orm(primary_key, auto_increment = false)]
    pub id_3: u8,
    #[sea_orm(primary_key, auto_increment = false)]
    pub id_4: i16,
    #[sea_orm(primary_key, auto_increment = false)]
    pub id_5: u16,
    #[sea_orm(primary_key, auto_increment = false)]
    pub id_6: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub id_7: u32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub id_8: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub id_9: u64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub id_10: f32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub id_11: f64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub id_12: bool,
    pub owner: String,
    pub name: String,
    pub description: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```
* Added macro `DerivePartialModel` https://github.com/SeaQL/sea-orm/pull/1597
```rust
#[derive(DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "Cake")]
struct PartialCake {
    name: String,
    #[sea_orm(
        from_expr = r#"SimpleExpr::FunctionCall(Func::upper(Expr::col((Cake, cake::Column::Name))))"#
    )]
    name_upper: String,
}

assert_eq!(
    cake::Entity::find()
        .into_partial_model::<PartialCake>()
        .into_statement(DbBackend::Sqlite)
        .to_string(),
    r#"SELECT "cake"."name", UPPER("cake"."name") AS "name_upper" FROM "cake""#
);
```
* [sea-orm-cli] Added support for generating migration of space separated name, for example executing `sea-orm-cli migrate generate "create accounts table"` command will create `m20230503_000000_create_accounts_table.rs` for you https://github.com/SeaQL/sea-orm/pull/1570

* Add `seaography` flag to `sea-orm`, `sea-orm-orm-macros` and `sea-orm-cli` https://github.com/SeaQL/sea-orm/pull/1599
* Add generation of `seaography` related information to `sea-orm-codegen` https://github.com/SeaQL/sea-orm/pull/1599

    The following information is added in entities files by `sea-orm-cli` when flag `seaography` is `true`
```rust
/// ... Entity File ...

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
    #[sea_orm(entity = "super::address::Entity")]
    Address,
    #[sea_orm(entity = "super::payment::Entity")]
    Payment,
    #[sea_orm(entity = "super::rental::Entity")]
    Rental,
    #[sea_orm(entity = "Entity", def = "Relation::SelfRef.def()")]
    SelfRef,
    #[sea_orm(entity = "super::store::Entity")]
    Store,
    #[sea_orm(entity = "Entity", def = "Relation::SelfRef.def().rev()")]
    SelfRefRev,
}
```
* Add `DeriveEntityRelated` macro https://github.com/SeaQL/sea-orm/pull/1599

    The DeriveRelatedEntity derive macro will implement `seaography::RelationBuilder` for `RelatedEntity` enumeration when the `seaography` feature is enabled

* Add `expr`, `exprs` and `expr_as` methods to `QuerySelect` trait https://github.com/SeaQL/sea-orm/pull/1702
```rust
use sea_orm::sea_query::Expr;
use sea_orm::{entity::*, tests_cfg::cake, DbBackend, QuerySelect, QueryTrait};

assert_eq!(
    cake::Entity::find()
        .select_only()
        .expr(Expr::col((cake::Entity, cake::Column::Id)))
        .build(DbBackend::MySql)
        .to_string(),
    "SELECT `cake`.`id` FROM `cake`"
);

assert_eq!(
    cake::Entity::find()
        .select_only()
        .exprs([
            Expr::col((cake::Entity, cake::Column::Id)),
            Expr::col((cake::Entity, cake::Column::Name)),
        ])
        .build(DbBackend::MySql)
        .to_string(),
    "SELECT `cake`.`id`, `cake`.`name` FROM `cake`"
);

assert_eq!(
    cake::Entity::find()
        .expr_as(
            Func::upper(Expr::col((cake::Entity, cake::Column::Name))),
            "name_upper"
        )
        .build(DbBackend::MySql)
        .to_string(),
    "SELECT `cake`.`id`, `cake`.`name`, UPPER(`cake`.`name`) AS `name_upper` FROM `cake`"
);
```
* Add `DbErr::sql_err()` method to convert error into common database errors `SqlErr`, such as unique constraint or foreign key violation errors. https://github.com/SeaQL/sea-orm/pull/1707
```rust
assert!(matches!(
    cake
        .into_active_model()
        .insert(db)
        .await
        .expect_err("Insert a row with duplicated primary key")
        .sql_err(),
    Some(SqlErr::UniqueConstraintViolation(_))
));

assert!(matches!(
    fk_cake
        .insert(db)
        .await
        .expect_err("Insert a row with invalid foreign key")
        .sql_err(),
    Some(SqlErr::ForeignKeyConstraintViolation(_))
));
```
* Add `Select::find_with_linked`, similar to `find_with_related`: https://github.com/SeaQL/sea-orm/pull/1728, https://github.com/SeaQL/sea-orm/pull/1743
```rust
fn find_with_related<R>(self, r: R) -> SelectTwoMany<E, R>
    where R: EntityTrait, E: Related<R>;
fn find_with_linked<L, T>(self, l: L) -> SelectTwoMany<E, T>
    where L: Linked<FromEntity = E, ToEntity = T>, T: EntityTrait;

// boths yields `Vec<(E::Model, Vec<F::Model>)>`
```
* Add `DeriveValueType` derive macro for custom wrapper types, implementations of the required traits will be provided, you can customize the `column_type` and `array_type` if needed https://github.com/SeaQL/sea-orm/pull/1720
```rust
#[derive(DeriveValueType)]
#[sea_orm(array_type = "Int")]
pub struct Integer(i32);

#[derive(DeriveValueType)]
#[sea_orm(column_type = "Boolean", array_type = "Bool")]
pub struct Boolbean(pub String);

#[derive(DeriveValueType)]
pub struct StringVec(pub Vec<String>);
```
The expanded code of `DeriveValueType` looks like.
```rust
#[derive(DeriveValueType)]
pub struct StringVec(pub Vec<String>);

// The `DeriveValueType` will be expanded into...

impl From<StringVec> for Value {
    fn from(source: StringVec) -> Self {
        source.0.into()
    }
}

impl sea_orm::TryGetable for StringVec {
    fn try_get_by<I: sea_orm::ColIdx>(res: &QueryResult, idx: I) -> Result<Self, sea_orm::TryGetError> {
        <Vec<String> as sea_orm::TryGetable>::try_get_by(res, idx).map(|v| StringVec(v))
    }
}

impl sea_orm::sea_query::ValueType for StringVec {
    fn try_from(v: Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
        <Vec<String> as sea_orm::sea_query::ValueType>::try_from(v).map(|v| StringVec(v))
    }

    fn type_name() -> String {
        stringify!(StringVec).to_owned()
    }

    fn array_type() -> sea_orm::sea_query::ArrayType {
        std::convert::Into::<sea_orm::sea_query::ArrayType>::into(
            <Vec<String> as sea_orm::sea_query::ValueType>::array_type()
        )
    }

    fn column_type() -> sea_orm::sea_query::ColumnType {
        std::convert::Into::<sea_orm::sea_query::ColumnType>::into(
            <Vec<String> as sea_orm::sea_query::ValueType>::column_type()
        )
    }
}
```
* Add `DeriveDisplay` derive macro to implements `std::fmt::Display` for active enum https://github.com/SeaQL/sea-orm/pull/1726
```rust
// String enum
#[derive(EnumIter, DeriveActiveEnum, DeriveDisplay)]
#[sea_orm(rs_type = "String", db_type = "String(Some(1))", enum_name = "category")]
pub enum DeriveCategory {
    #[sea_orm(string_value = "B")]
    Big,
    #[sea_orm(string_value = "S")]
    Small,
}
assert_eq!(format!("{}", DeriveCategory::Big), "Big");
assert_eq!(format!("{}", DeriveCategory::Small), "Small");

// Numeric enum
#[derive(EnumIter, DeriveActiveEnum, DeriveDisplay)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum $ident {
    #[sea_orm(num_value = -10)]
    Negative,
    #[sea_orm(num_value = 1)]
    Big,
    #[sea_orm(num_value = 0)]
    Small,
}
assert_eq!(format!("{}", $ident::Big), "Big");
assert_eq!(format!("{}", $ident::Small), "Small");
assert_eq!(format!("{}", $ident::Negative), "Negative");

// String enum with `display_value` overrides
#[derive(EnumIter, DeriveActiveEnum, DeriveDisplay)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "tea")]
pub enum DisplayTea {
    #[sea_orm(string_value = "EverydayTea", display_value = "Everyday")]
    EverydayTea,
    #[sea_orm(string_value = "BreakfastTea", display_value = "Breakfast")]
    BreakfastTea,
}
assert_eq!(format!("{}", DisplayTea::BreakfastTea), "Breakfast");
assert_eq!(format!("{}", DisplayTea::EverydayTea), "Everyday");
```

### Enhancements

* Added `Migration::name()` and `Migration::status()` getters for the name and status of `sea_orm_migration::Migration` https://github.com/SeaQL/sea-orm/pull/1519
```rust
let migrations = Migrator::get_pending_migrations(db).await?;
assert_eq!(migrations.len(), 5);

let migration = migrations.get(0).unwrap();
assert_eq!(migration.name(), "m20220118_000002_create_fruit_table");
assert_eq!(migration.status(), MigrationStatus::Pending);
```
* The `postgres-array` feature will be enabled when `sqlx-postgres` backend is selected https://github.com/SeaQL/sea-orm/pull/1565
* Replace `String` parameters in API with `Into<String>` https://github.com/SeaQL/sea-orm/pull/1439
    * Implements `IntoMockRow` for any `BTreeMap` that is indexed by string `impl IntoMockRow for BTreeMap<T, Value> where T: Into<String>`
    * Converts any string value into `ConnectOptions` - `impl From<T> for ConnectOptions where T: Into<String>`
    * Changed the parameter of method `ConnectOptions::new(T) where T: Into<String>` to takes any string SQL
    * Changed the parameter of method `Statement::from_string(DbBackend, T) where T: Into<String>` to takes any string SQL
    * Changed the parameter of method `Statement::from_sql_and_values(DbBackend, T, I) where I: IntoIterator<Item = Value>, T: Into<String>` to takes any string SQL
    * Changed the parameter of method `Transaction::from_sql_and_values(DbBackend, T, I) where I: IntoIterator<Item = Value>, T: Into<String>` to takes any string SQL
    * Changed the parameter of method `ConnectOptions::set_schema_search_path(T) where T: Into<String>` to takes any string
    * Changed the parameter of method `ColumnTrait::like()`, `ColumnTrait::not_like()`, `ColumnTrait::starts_with()`, `ColumnTrait::ends_with()` and `ColumnTrait::contains()` to takes any string
* Re-export `sea_query::{DynIden, RcOrArc, SeaRc}` in `sea_orm::entity::prelude` module https://github.com/SeaQL/sea-orm/pull/1661
* Added `DatabaseConnection::ping` https://github.com/SeaQL/sea-orm/pull/1627
```rust
|db: DatabaseConnection| {
    assert!(db.ping().await.is_ok());
    db.clone().close().await;
    assert!(matches!(db.ping().await, Err(DbErr::ConnectionAcquire)));
}
```
* Added `TryInsert` that does not panic on empty inserts https://github.com/SeaQL/sea-orm/pull/1708
```rust
// now, you can do:
let res = Bakery::insert_many(std::iter::empty())
    .on_empty_do_nothing()
    .exec(db)
    .await;

assert!(matches!(res, Ok(TryInsertResult::Empty)));
```
* On conflict do nothing not resulting in Err https://github.com/SeaQL/sea-orm/pull/1712
```rust
let on = OnConflict::column(Column::Id).do_nothing().to_owned();

// Existing behaviour
let res = Entity::insert_many([..]).on_conflict(on).exec(db).await;
assert!(matches!(res, Err(DbErr::RecordNotInserted)));

// New API; now you can:
let res = Entity::insert_many([..]).on_conflict(on).do_nothing().exec(db).await;
assert!(matches!(res, Ok(TryInsertResult::Conflicted)));
```
* Added `UpdateMany::exec_with_returning()` https://github.com/SeaQL/sea-orm/pull/1677
```rust
Entity::update_many()
    .col_expr(Column::Values, Expr::expr(..))
    .exec_with_returning(db)
    .await
```

### Upgrades

* Upgrade `heck` dependency in `sea-orm-macros` and `sea-orm-codegen` to 0.4 https://github.com/SeaQL/sea-orm/pull/1520, https://github.com/SeaQL/sea-orm/pull/1544
* Upgrade `strum` to 0.25 https://github.com/SeaQL/sea-orm/pull/1752
* Upgrade `sea-query` to 0.29 https://github.com/SeaQL/sea-orm/pull/1562
* Upgrade `sea-query-binder` to 0.4 https://github.com/SeaQL/sea-orm/pull/1562
* Upgrade `sea-schema` to 0.12 https://github.com/SeaQL/sea-orm/pull/1562
* Upgrade `clap` to 4.3 https://github.com/SeaQL/sea-orm/pull/1468
* Replace `bae` with `sea-bae` https://github.com/SeaQL/sea-orm/pull/1739

### Bug Fixes

* Fixed `DeriveActiveEnum` throwing errors because `string_value` consists non-UAX#31 compliant characters https://github.com/SeaQL/sea-orm/pull/1374

For example,
```rust
#[derive(Clone, Debug, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(None)")]
pub enum StringValue {
    #[sea_orm(string_value = "")]
    Member1,
    #[sea_orm(string_value = "$")]
    Member2,
    #[sea_orm(string_value = "$$")]
    Member3,
    #[sea_orm(string_value = "AB")]
    Member4,
    #[sea_orm(string_value = "A_B")]
    Member5,
    #[sea_orm(string_value = "A$B")]
    Member6,
    #[sea_orm(string_value = "0 123")]
    Member7,
}
```
will now produce the following Variant Enum:
```rust
pub enum StringValueVariant {
    __Empty,
    _0x24,
    _0x240x24,
    Ab,
    A0x5Fb,
    A0x24B,
    _0x300x20123,
}
```
* [sea-orm-cli] The implementation of `Related<R>` with `via` and `to` methods will not be generated if there exists multiple paths via an intermediate table. Like in the schema defined below - Path 1. `users <-> users_votes <-> bills`, Path 2. `users <-> users_saved_bills <-> bills` https://github.com/SeaQL/sea-orm/pull/1435
```sql
CREATE TABLE users
(
  id uuid  PRIMARY KEY  DEFAULT uuid_generate_v1mc(),
  email TEXT UNIQUE NOT NULL,
  ...
);
```
```sql
CREATE TABLE bills
(
  id uuid  PRIMARY KEY  DEFAULT uuid_generate_v1mc(),
  ...
);
```
```sql
CREATE TABLE users_votes
(
  user_id uuid REFERENCES users (id) ON UPDATE CASCADE ON DELETE CASCADE,
  bill_id uuid REFERENCES bills (id) ON UPDATE CASCADE ON DELETE CASCADE,
  vote boolean NOT NULL,
  CONSTRAINT users_bills_pkey PRIMARY KEY (user_id, bill_id)
);
```
```sql
CREATE TABLE users_saved_bills
(
  user_id uuid REFERENCES users (id) ON UPDATE CASCADE ON DELETE CASCADE,
  bill_id uuid REFERENCES bills (id) ON UPDATE CASCADE ON DELETE CASCADE,
  CONSTRAINT users_saved_bills_pkey PRIMARY KEY (user_id, bill_id)
);
```
* [sea-orm-cli] fixed entity generation includes partitioned tables https://github.com/SeaQL/sea-orm/issues/1582, https://github.com/SeaQL/sea-schema/pull/105
* Fixed `ActiveEnum::db_type()` return type does not implement `ColumnTypeTrait` https://github.com/SeaQL/sea-orm/pull/1576
```rust
impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
...
            // `db_type()` returns `ColumnDef`; now it implements `ColumnTypeTrait`
            Self::Thing => AnActiveEnumThing::db_type().def(),
...
        }
    }
}
```
* Resolved `insert_many` failing if the models iterator is empty https://github.com/SeaQL/sea-orm/issues/873
* Update the template MD file of `migration/README.md`, fix a faulty sample `migrate init` shell script https://github.com/SeaQL/sea-orm/pull/1723

### Breaking changes

* Supports for partial select of `Option<T>` model field. A `None` value will be filled when the select result does not contain the `Option<T>` field instead of throwing an error. https://github.com/SeaQL/sea-orm/pull/1513
* Replaced `sea-strum` dependency with upstream `strum` in `sea-orm` https://github.com/SeaQL/sea-orm/pull/1535
    * Added `derive` and `strum` features to `sea-orm-macros`
    * The derive macro `EnumIter` is now shipped by `sea-orm-macros`
* Added a new variant `Many` to `Identity` https://github.com/SeaQL/sea-orm/pull/1508
* Replace the use of `SeaRc<T>` where `T` isn't `dyn Iden` with `RcOrArc<T>` https://github.com/SeaQL/sea-orm/pull/1661
* Enabled `hashable-value` feature in SeaQuery, thus `Value::Float(NaN) == Value::Float(NaN)` would be true https://github.com/SeaQL/sea-orm/pull/1728, https://github.com/SeaQL/sea-orm/pull/1743
* The `DeriveActiveEnum` derive macro no longer provide `std::fmt::Display` implementation for the enum. You need to derive an extra `DeriveDisplay` macro alongside with `DeriveActiveEnum` derive macro. https://github.com/SeaQL/sea-orm/pull/1726

## 0.11.3 - 2023-04-24

### Enhancements

* Re-export `sea_orm::ConnectionTrait` in `sea_orm_migration::prelude` https://github.com/SeaQL/sea-orm/pull/1577
* Support generic structs in `FromQueryResult` derive macro https://github.com/SeaQL/sea-orm/pull/1464, https://github.com/SeaQL/sea-orm/pull/1603
```rust
#[derive(FromQueryResult)]
struct GenericTest<T: TryGetable> {
    foo: i32,
    bar: T,
}
```
```rust
trait MyTrait {
    type Item: TryGetable;
}

#[derive(FromQueryResult)]
struct TraitAssociateTypeTest<T>
where
    T: MyTrait,
{
    foo: T::Item,
}
```

### Bug Fixes

* Fixed https://github.com/SeaQL/sea-orm/issues/1608 by pinning the version of `tracing-subscriber` dependency to 0.3.17 https://github.com/SeaQL/sea-orm/pull/1609

## 0.11.2 - 2023-03-25

### Enhancements

* Enable required `syn` features https://github.com/SeaQL/sea-orm/pull/1556
* Re-export `sea_query::BlobSize` in `sea_orm::entity::prelude` https://github.com/SeaQL/sea-orm/pull/1548

## 0.11.1 - 2023-03-10

### Bug Fixes

* Fixes `DeriveActiveEnum` (by qualifying `ColumnTypeTrait::def`) https://github.com/SeaQL/sea-orm/issues/1478
* The CLI command `sea-orm-cli generate entity -u '<DB-URL>'` will now generate the following code for each `Binary` or `VarBinary` columns in compact format https://github.com/SeaQL/sea-orm/pull/1529
```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "binary")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))")]
    pub binary: Vec<u8>,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(Some(10)))")]
    pub binary_10: Vec<u8>,
    #[sea_orm(column_type = "Binary(BlobSize::Tiny)")]
    pub binary_tiny: Vec<u8>,
    #[sea_orm(column_type = "Binary(BlobSize::Medium)")]
    pub binary_medium: Vec<u8>,
    #[sea_orm(column_type = "Binary(BlobSize::Long)")]
    pub binary_long: Vec<u8>,
    #[sea_orm(column_type = "VarBinary(10)")]
    pub var_binary: Vec<u8>,
}
```
* The CLI command `sea-orm-cli generate entity -u '<DB-URL>' --expanded-format` will now generate the following code for each `Binary` or `VarBinary` columns in expanded format https://github.com/SeaQL/sea-orm/pull/1529
```rust
impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::Id => ColumnType::Integer.def(),
            Self::Binary => ColumnType::Binary(sea_orm::sea_query::BlobSize::Blob(None)).def(),
            Self::Binary10 => {
                ColumnType::Binary(sea_orm::sea_query::BlobSize::Blob(Some(10u32))).def()
            }
            Self::BinaryTiny => ColumnType::Binary(sea_orm::sea_query::BlobSize::Tiny).def(),
            Self::BinaryMedium => ColumnType::Binary(sea_orm::sea_query::BlobSize::Medium).def(),
            Self::BinaryLong => ColumnType::Binary(sea_orm::sea_query::BlobSize::Long).def(),
            Self::VarBinary => ColumnType::VarBinary(10u32).def(),
        }
    }
}
```
* Fix missing documentation on type generated by derive macros https://github.com/SeaQL/sea-orm/pull/1522, https://github.com/SeaQL/sea-orm/pull/1531

## 0.11.0 - 2023-02-07

+ 2023-02-02: `0.11.0-rc.1`
+ 2023-02-04: `0.11.0-rc.2`

### New Features

#### SeaORM Core

* Simple data loader https://github.com/SeaQL/sea-orm/pull/1238, https://github.com/SeaQL/sea-orm/pull/1443
* Transactions Isolation level and Access mode https://github.com/SeaQL/sea-orm/pull/1230
* Support various UUID formats that are available in `uuid::fmt` module https://github.com/SeaQL/sea-orm/pull/1325
* Support Vector of enum for Postgres https://github.com/SeaQL/sea-orm/pull/1210
* Support `ActiveEnum` field as primary key https://github.com/SeaQL/sea-orm/pull/1414
* Casting columns as a different data type on select, insert and update https://github.com/SeaQL/sea-orm/pull/1304
* Methods of `ActiveModelBehavior` receive db connection as a parameter https://github.com/SeaQL/sea-orm/pull/1145, https://github.com/SeaQL/sea-orm/pull/1328
* Added `execute_unprepared` method to `DatabaseConnection` and `DatabaseTransaction` https://github.com/SeaQL/sea-orm/pull/1327
* Added `Select::into_tuple` to select rows as tuples (instead of defining a custom Model) https://github.com/SeaQL/sea-orm/pull/1311

#### SeaORM CLI

* Generate `#[serde(skip_deserializing)]` for primary key columns https://github.com/SeaQL/sea-orm/pull/846, https://github.com/SeaQL/sea-orm/pull/1186, https://github.com/SeaQL/sea-orm/pull/1318
* Generate `#[serde(skip)]` for hidden columns https://github.com/SeaQL/sea-orm/pull/1171, https://github.com/SeaQL/sea-orm/pull/1320
* Generate entity with extra derives and attributes for model struct https://github.com/SeaQL/sea-orm/pull/1124, https://github.com/SeaQL/sea-orm/pull/1321

#### SeaORM Migration

* Migrations are now performed inside a transaction for Postgres https://github.com/SeaQL/sea-orm/pull/1379

### Enhancements

* Refactor schema module to expose functions for database alteration https://github.com/SeaQL/sea-orm/pull/1256
* Generate compact entity with `#[sea_orm(column_type = "JsonBinary")]` macro attribute https://github.com/SeaQL/sea-orm/pull/1346
* `MockDatabase::append_exec_results()`, `MockDatabase::append_query_results()`, `MockDatabase::append_exec_errors()` and `MockDatabase::append_query_errors()` take any types implemented `IntoIterator` trait https://github.com/SeaQL/sea-orm/pull/1367
* `find_by_id` and `delete_by_id` take any `Into` primary key value https://github.com/SeaQL/sea-orm/pull/1362
* `QuerySelect::offset` and `QuerySelect::limit` takes in `Into<Option<u64>>` where `None` would reset them https://github.com/SeaQL/sea-orm/pull/1410
* Added `DatabaseConnection::close` https://github.com/SeaQL/sea-orm/pull/1236
* Added `is_null` getter for `ColumnDef` https://github.com/SeaQL/sea-orm/pull/1381
* Added `ActiveValue::reset` to convert `Unchanged` into `Set` https://github.com/SeaQL/sea-orm/pull/1177
* Added `QueryTrait::apply_if` to optionally apply a filter https://github.com/SeaQL/sea-orm/pull/1415
* Added the `sea-orm-internal` feature flag to expose some SQLx types
    * Added `DatabaseConnection::get_*_connection_pool()` for accessing the inner SQLx connection pool https://github.com/SeaQL/sea-orm/pull/1297
    * Re-exporting SQLx errors https://github.com/SeaQL/sea-orm/pull/1434

### Upgrades

* Upgrade `axum` to `0.6.1` https://github.com/SeaQL/sea-orm/pull/1285
* Upgrade `sea-query` to `0.28` https://github.com/SeaQL/sea-orm/pull/1366
* Upgrade `sea-query-binder` to `0.3` https://github.com/SeaQL/sea-orm/pull/1366
* Upgrade `sea-schema` to `0.11` https://github.com/SeaQL/sea-orm/pull/1366

### House Keeping

* Fixed all clippy warnings as of `1.67.0` https://github.com/SeaQL/sea-orm/pull/1426
* Removed dependency where not needed https://github.com/SeaQL/sea-orm/pull/1213
* Disabled default features and enabled only the needed ones https://github.com/SeaQL/sea-orm/pull/1300
* Cleanup panic and unwrap https://github.com/SeaQL/sea-orm/pull/1231
* Cleanup the use of `vec!` macro https://github.com/SeaQL/sea-orm/pull/1367
* Upgrade `syn` to v2 https://github.com/SeaQL/sea-orm/pull/1713
* Upgrade `ouroboros` to `0.17` https://github.com/SeaQL/sea-orm/pull/1724

### Bug Fixes

* [sea-orm-cli] Propagate error on the spawned child processes https://github.com/SeaQL/sea-orm/pull/1402
    * Fixes sea-orm-cli errors exit with error code 0 https://github.com/SeaQL/sea-orm/issues/1342
* Fixes `DeriveColumn` (by qualifying `IdenStatic::as_str`) https://github.com/SeaQL/sea-orm/pull/1280
* Prevent returning connections to pool with a positive transaction depth https://github.com/SeaQL/sea-orm/pull/1283
* Postgres insert many will throw `RecordNotInserted` error if non of them are being inserted https://github.com/SeaQL/sea-orm/pull/1021
    * Fixes inserting active models by `insert_many` with `on_conflict` and `do_nothing` panics if no rows are inserted on Postgres https://github.com/SeaQL/sea-orm/issues/899
* Don't call `last_insert_id` if not needed https://github.com/SeaQL/sea-orm/pull/1403
    * Fixes hitting 'negative last_insert_rowid' panic with Sqlite https://github.com/SeaQL/sea-orm/issues/1357
* Noop when update without providing any values https://github.com/SeaQL/sea-orm/pull/1384
    * Fixes Syntax Error when saving active model that sets nothing https://github.com/SeaQL/sea-orm/pull/1376

### Breaking Changes

* [sea-orm-cli] Enable --universal-time by default https://github.com/SeaQL/sea-orm/pull/1420
* Added `RecordNotInserted` and `RecordNotUpdated` to `DbErr`
* Added `ConnectionTrait::execute_unprepared` method https://github.com/SeaQL/sea-orm/pull/1327
* As part of https://github.com/SeaQL/sea-orm/pull/1311, the required method of `TryGetable` changed:
```rust
// then
fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError>;
// now; ColIdx can be `&str` or `usize`
fn try_get_by<I: ColIdx>(res: &QueryResult, index: I) -> Result<Self, TryGetError>;
```
So if you implemented it yourself:
```patch
impl TryGetable for XXX {
-   fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
+   fn try_get_by<I: sea_orm::ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
-       let value: YYY = res.try_get(pre, col).map_err(TryGetError::DbErr)?;
+       let value: YYY = res.try_get_by(idx).map_err(TryGetError::DbErr)?;
        ..
    }
}
```
* The `ActiveModelBehavior` trait becomes async trait https://github.com/SeaQL/sea-orm/pull/1328.
If you overridden the default `ActiveModelBehavior` implementation:
```rust
#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        // ...
    }

    // ...
}
```
* `DbErr::RecordNotFound("None of the database rows are affected")` is moved to a dedicated error variant `DbErr::RecordNotUpdated` https://github.com/SeaQL/sea-orm/pull/1425
```rust
let res = Update::one(cake::ActiveModel {
        name: Set("Cheese Cake".to_owned()),
        ..model.into_active_model()
    })
    .exec(&db)
    .await;

// then
assert_eq!(
    res,
    Err(DbErr::RecordNotFound(
        "None of the database rows are affected".to_owned()
    ))
);

// now
assert_eq!(res, Err(DbErr::RecordNotUpdated));
```
* `sea_orm::ColumnType` was replaced by `sea_query::ColumnType` https://github.com/SeaQL/sea-orm/pull/1395
    * Method `ColumnType::def` was moved to `ColumnTypeTrait`
    * `ColumnType::Binary` becomes a tuple variant which takes in additional option `sea_query::BlobSize`
    * `ColumnType::Custom` takes a `sea_query::DynIden` instead of `String` and thus a new method `custom` is added (note the lowercase)
```diff
// Compact Entity
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "fruit")]
pub struct Model {
-   #[sea_orm(column_type = r#"Custom("citext".to_owned())"#)]
+   #[sea_orm(column_type = r#"custom("citext")"#)]
    pub column: String,
}
```
```diff
// Expanded Entity
impl ColumnTrait for Column {
    type EntityName = Entity;

    fn def(&self) -> ColumnDef {
        match self {
-           Self::Column => ColumnType::Custom("citext".to_owned()).def(),
+           Self::Column => ColumnType::custom("citext").def(),
        }
    }
}
```

### Miscellaneous

* Fixed a small typo https://github.com/SeaQL/sea-orm/pull/1391
* `axum` example should use tokio runtime https://github.com/SeaQL/sea-orm/pull/1428

Full Changelog: https://github.com/SeaQL/sea-orm/compare/0.10.0...0.11.0

## 0.10.7 - 2023-01-19

### Bug Fixes

* Inserting active models by `insert_many` with `on_conflict` and `do_nothing` panics if no rows are inserted on Postgres https://github.com/SeaQL/sea-orm/issues/899
* Hitting 'negative last_insert_rowid' panic with Sqlite https://github.com/SeaQL/sea-orm/issues/1357

## 0.10.6 - 2022-12-23

### Enhancements

* Cast enum values when constructing update many query https://github.com/SeaQL/sea-orm/pull/1178

### Bug Fixes

* Fixes `DeriveColumn` (by qualifying `IdenStatic::as_str`) https://github.com/SeaQL/sea-orm/pull/1280
* Prevent returning connections to pool with a positive transaction depth https://github.com/SeaQL/sea-orm/pull/1283
* [sea-orm-codegen] Skip implementing Related if the same related entity is being referenced by a conjunct relation https://github.com/SeaQL/sea-orm/pull/1298
* [sea-orm-cli] CLI depends on codegen of the same version https://github.com/SeaQL/sea-orm/pull/1299/

## 0.10.5 - 2022-12-02

### New Features

* Add `QuerySelect::columns` method - select multiple columns https://github.com/SeaQL/sea-orm/pull/1264
* Transactions Isolation level and Access mode https://github.com/SeaQL/sea-orm/pull/1230

### Bug Fixes

* `DeriveEntityModel` derive macro: when parsing field type, always treat field with `Option<T>` as nullable column https://github.com/SeaQL/sea-orm/pull/1257

### Enhancements

* [sea-orm-cli] Generate `Related` implementation for many-to-many relation with extra columns https://github.com/SeaQL/sea-orm/pull/1260
* Optimize the default implementation of `TryGetableFromJson::try_get_from_json()` - deserializing into `Self` directly without the need of a intermediate `serde_json::Value` https://github.com/SeaQL/sea-orm/pull/1249

## 0.10.4 - 2022-11-24

### Bug Fixes

* Fix DeriveActiveEnum expand enum variant starts with number https://github.com/SeaQL/sea-orm/pull/1219
* [sea-orm-cli] Generate entity file for specified tables only https://github.com/SeaQL/sea-orm/pull/1245
* Support appending `DbErr` to `MockDatabase` https://github.com/SeaQL/sea-orm/pull/1241

### Enhancements

* Filter rows with `IS IN` enum values expression https://github.com/SeaQL/sea-orm/pull/1183
* [sea-orm-cli] Generate entity with relation variant order by name of reference table https://github.com/SeaQL/sea-orm/pull/1229

## 0.10.3 - 2022-11-14

### Bug Fixes

* [sea-orm-cli] Set search path when initializing Postgres connection for CLI generate entity https://github.com/SeaQL/sea-orm/pull/1212
* [sea-orm-cli] Generate `_` prefix to enum variant starts with number https://github.com/SeaQL/sea-orm/pull/1211
* Fix composite key cursor pagination https://github.com/SeaQL/sea-orm/pull/1216
    + The logic for single-column primary key was correct, but for composite keys the logic was incorrect

### Enhancements

* Added `Insert::exec_without_returning` https://github.com/SeaQL/sea-orm/pull/1208

### House Keeping

* Remove dependency when not needed https://github.com/SeaQL/sea-orm/pull/1207

## 0.10.2 - 2022-11-06

### Enhancements

* [sea-orm-rocket] added `sqlx_logging` to `Config` https://github.com/SeaQL/sea-orm/pull/1192
* Collecting metrics for `query_one/all` https://github.com/SeaQL/sea-orm/pull/1165
* Use GAT to elide `StreamTrait` lifetime https://github.com/SeaQL/sea-orm/pull/1161

### Bug Fixes

* corrected the error name `UpdateGetPrimaryKey` https://github.com/SeaQL/sea-orm/pull/1180

### Upgrades

* Update MSRV to 1.65

## 0.10.1 - 2022-10-27

### Enhancements

* [sea-orm-cli] Escape module name defined with Rust keywords https://github.com/SeaQL/sea-orm/pull/1052
* [sea-orm-cli] Check to make sure migration name doesn't contain hyphen `-` in it https://github.com/SeaQL/sea-orm/pull/879, https://github.com/SeaQL/sea-orm/pull/1155
* Support `time` crate for SQLite https://github.com/SeaQL/sea-orm/pull/995

### Bug Fixes

* [sea-orm-cli] Generate `Related` for m-to-n relation https://github.com/SeaQL/sea-orm/pull/1075
* [sea-orm-cli] Generate model entity with Postgres Enum field https://github.com/SeaQL/sea-orm/pull/1153
* [sea-orm-cli] Migrate up command apply all pending migrations https://github.com/SeaQL/sea-orm/pull/1010
* [sea-orm-cli] Conflicting short flag `-u` when executing `migrate generate` command https://github.com/SeaQL/sea-orm/pull/1157
* Prefix the usage of types with `sea_orm::` inside `DeriveActiveEnum` derive macros https://github.com/SeaQL/sea-orm/pull/1146, https://github.com/SeaQL/sea-orm/pull/1154
* [sea-orm-cli] Generate model with `Vec<f32>` or `Vec<f64>` should not derive `Eq` on the model struct https://github.com/SeaQL/sea-orm/pull/1158

### House Keeping

* [sea-orm-cli] [sea-orm-migration] Add `cli` feature to optionally include dependencies that are required by the CLI https://github.com/SeaQL/sea-orm/pull/978

### Upgrades

* Upgrade `sea-schema` to 0.10.2 https://github.com/SeaQL/sea-orm/pull/1153

## 0.10.0 - 2022-10-23

### New Features

* Better error types (carrying SQLx Error) https://github.com/SeaQL/sea-orm/pull/1002
* Support array datatype in PostgreSQL https://github.com/SeaQL/sea-orm/pull/1132
* [sea-orm-cli] Generate entity files as a library or module https://github.com/SeaQL/sea-orm/pull/953
* [sea-orm-cli] Generate a new migration template with name prefix of unix timestamp https://github.com/SeaQL/sea-orm/pull/947
* [sea-orm-cli] Generate migration in modules https://github.com/SeaQL/sea-orm/pull/933
* [sea-orm-cli] Generate `DeriveRelation` on empty `Relation` enum https://github.com/SeaQL/sea-orm/pull/1019
* [sea-orm-cli] Generate entity derive `Eq` if possible https://github.com/SeaQL/sea-orm/pull/988
* [sea-orm-cli] Run migration on any PostgreSQL schema https://github.com/SeaQL/sea-orm/pull/1056

### Enhancements

* Support `distinct` & `distinct_on` expression https://github.com/SeaQL/sea-orm/pull/902
* `fn column()` also handle enum type https://github.com/SeaQL/sea-orm/pull/973
* Added `acquire_timeout` on `ConnectOptions` https://github.com/SeaQL/sea-orm/pull/897
* [sea-orm-cli] `migrate fresh` command will drop all PostgreSQL types https://github.com/SeaQL/sea-orm/pull/864, https://github.com/SeaQL/sea-orm/pull/991
* Better compile error for entity without primary key https://github.com/SeaQL/sea-orm/pull/1020
* Added blanket implementations of `IntoActiveValue` for `Option` values https://github.com/SeaQL/sea-orm/pull/833
* Added `into_model` & `into_json` to `Cursor` https://github.com/SeaQL/sea-orm/pull/1112
* Added `set_schema_search_path` method to `ConnectOptions` for setting schema search path of PostgreSQL connection https://github.com/SeaQL/sea-orm/pull/1056
* Serialize `time` types as `serde_json::Value` https://github.com/SeaQL/sea-orm/pull/1042
* Implements `fmt::Display` for `ActiveEnum` https://github.com/SeaQL/sea-orm/pull/986
* Implements `TryFrom<ActiveModel>` for `Model` https://github.com/SeaQL/sea-orm/pull/990

### Bug Fixes

* Trim spaces when paginating raw SQL https://github.com/SeaQL/sea-orm/pull/1094

### Breaking Changes

* Replaced `usize` with `u64` in `PaginatorTrait` https://github.com/SeaQL/sea-orm/pull/789
* Type signature of `DbErr` changed as a result of https://github.com/SeaQL/sea-orm/pull/1002
* `ColumnType::Enum` structure changed:
```rust
enum ColumnType {
    // then
    Enum(String, Vec<String>)

    // now
    Enum {
        /// Name of enum
        name: DynIden,
        /// Variants of enum
        variants: Vec<DynIden>,
    }
    ...
}

// example

#[derive(Iden)]
enum TeaEnum {
    #[iden = "tea"]
    Enum,
    #[iden = "EverydayTea"]
    EverydayTea,
    #[iden = "BreakfastTea"]
    BreakfastTea,
}

// then
ColumnDef::new(active_enum_child::Column::Tea)
    .enumeration("tea", vec!["EverydayTea", "BreakfastTea"])

// now
ColumnDef::new(active_enum_child::Column::Tea)
    .enumeration(TeaEnum::Enum, [TeaEnum::EverydayTea, TeaEnum::BreakfastTea])
```

* A new method `array_type` was added to `ValueType`:
```rust
impl sea_orm::sea_query::ValueType for MyType {
    fn array_type() -> sea_orm::sea_query::ArrayType {
        sea_orm::sea_query::ArrayType::TypeName
    }
    ...
}
```

* `ActiveEnum::name()` changed return type to `DynIden`:
```rust
#[derive(Debug, Iden)]
#[iden = "category"]
pub struct CategoryEnum;

impl ActiveEnum for Category {
    // then
    fn name() -> String {
        "category".to_owned()
    }

    // now
    fn name() -> DynIden {
        SeaRc::new(CategoryEnum)
    }
    ...
}
```

### House Keeping

* Documentation grammar fixes https://github.com/SeaQL/sea-orm/pull/1050
* Replace `dotenv` with `dotenvy` in examples https://github.com/SeaQL/sea-orm/pull/1085
* Exclude test_cfg module from SeaORM https://github.com/SeaQL/sea-orm/pull/1077

### Integration

* Support `rocket_okapi` https://github.com/SeaQL/sea-orm/pull/1071

### Upgrades

* Upgrade `sea-query` to 0.26 https://github.com/SeaQL/sea-orm/pull/985

**Full Changelog**: https://github.com/SeaQL/sea-orm/compare/0.9.0...0.10.0

## 0.9.3 - 2022-09-30

### Enhancements

* `fn column()` also handle enum type https://github.com/SeaQL/sea-orm/pull/973
* Generate migration in modules https://github.com/SeaQL/sea-orm/pull/933
* Generate `DeriveRelation` on empty `Relation` enum https://github.com/SeaQL/sea-orm/pull/1019
* Documentation grammar fixes https://github.com/SeaQL/sea-orm/pull/1050

### Bug Fixes

* Implement `IntoActiveValue` for `time` types https://github.com/SeaQL/sea-orm/pull/1041
* Fixed module import for `FromJsonQueryResult` derive macro https://github.com/SeaQL/sea-orm/pull/1081

## 0.9.2 - 2022-08-20

### Enhancements

* [sea-orm-cli] Migrator CLI handles init and generate commands https://github.com/SeaQL/sea-orm/pull/931
* [sea-orm-cli] added `with-copy-enums` flag to conditional derive `Copy` on `ActiveEnum` https://github.com/SeaQL/sea-orm/pull/936

### House Keeping

* Exclude `chrono` default features https://github.com/SeaQL/sea-orm/pull/950
* Set minimal rustc version to `1.60` https://github.com/SeaQL/sea-orm/pull/938
* Update `sea-query` to `0.26.3`

### Notes

In this minor release, we removed `time` v0.1 from the dependency graph

## 0.9.1 - 2022-07-22

### Enhancements

* [sea-orm-cli] Codegen support for `VarBinary` column type https://github.com/SeaQL/sea-orm/pull/746
* [sea-orm-cli] Generate entity for SYSTEM VERSIONED tables on MariaDB https://github.com/SeaQL/sea-orm/pull/876

### Bug Fixes

* `RelationDef` & `RelationBuilder` should be `Send` & `Sync` https://github.com/SeaQL/sea-orm/pull/898

### House Keeping

* Remove unnecessary `async_trait` https://github.com/SeaQL/sea-orm/pull/737

## 0.9.0 - 2022-07-17

### New Features

* Cursor pagination https://github.com/SeaQL/sea-orm/pull/822
* Custom join on conditions https://github.com/SeaQL/sea-orm/pull/793
* `DeriveMigrationName` and `sea_orm_migration::util::get_file_stem` https://github.com/SeaQL/sea-orm/pull/736
* `FromJsonQueryResult` for deserializing `Json` from query result https://github.com/SeaQL/sea-orm/pull/794

### Enhancements

* Added `sqlx_logging_level` to `ConnectOptions` https://github.com/SeaQL/sea-orm/pull/800
* Added `num_items_and_pages` to `Paginator` https://github.com/SeaQL/sea-orm/pull/768
* Added `TryFromU64` for `time` https://github.com/SeaQL/sea-orm/pull/849
* Added `Insert::on_conflict` https://github.com/SeaQL/sea-orm/pull/791
* Added `QuerySelect::join_as` and `QuerySelect::join_as_rev` https://github.com/SeaQL/sea-orm/pull/852
* Include column name in `TryGetError::Null` https://github.com/SeaQL/sea-orm/pull/853
* [sea-orm-cli] Improve logging https://github.com/SeaQL/sea-orm/pull/735
* [sea-orm-cli] Generate enum with numeric like variants https://github.com/SeaQL/sea-orm/pull/588
* [sea-orm-cli] Allow old pending migration to be applied https://github.com/SeaQL/sea-orm/pull/755
* [sea-orm-cli] Skip generating entity for ignored tables https://github.com/SeaQL/sea-orm/pull/837
* [sea-orm-cli] Generate code for `time` crate https://github.com/SeaQL/sea-orm/pull/724
* [sea-orm-cli] Add various blob column types https://github.com/SeaQL/sea-orm/pull/850
* [sea-orm-cli] Generate entity files with Postgres's schema name https://github.com/SeaQL/sea-orm/pull/422

### Upgrades

* Upgrade `clap` to 3.2 https://github.com/SeaQL/sea-orm/pull/706
* Upgrade `time` to 0.3 https://github.com/SeaQL/sea-orm/pull/834
* Upgrade `sqlx` to 0.6 https://github.com/SeaQL/sea-orm/pull/834
* Upgrade `uuid` to 1.0 https://github.com/SeaQL/sea-orm/pull/834
* Upgrade `sea-query` to 0.26 https://github.com/SeaQL/sea-orm/pull/834
* Upgrade `sea-schema` to 0.9 https://github.com/SeaQL/sea-orm/pull/834

### House Keeping

* Refactor stream metrics https://github.com/SeaQL/sea-orm/pull/778

### Bug Fixes

* [sea-orm-cli] skip checking connection string for credentials https://github.com/SeaQL/sea-orm/pull/851

### Breaking Changes

* `SelectTwoMany::one()` has been dropped https://github.com/SeaQL/sea-orm/pull/813, you can get `(Entity, Vec<RelatedEntity>)` by first querying a single model from Entity, then use [`ModelTrait::find_related`] on the model.
* #### Feature flag revamp
    We now adopt the [weak dependency](https://blog.rust-lang.org/2022/04/07/Rust-1.60.0.html#new-syntax-for-cargo-features) syntax in Cargo. That means the flags `["sqlx-json", "sqlx-chrono", "sqlx-decimal", "sqlx-uuid", "sqlx-time"]` are not needed and now removed. Instead, `with-time` will enable `sqlx?/time` only if `sqlx` is already enabled. As a consequence, now the features `with-json`, `with-chrono`, `with-rust_decimal`, `with-uuid`, `with-time` will not be enabled as a side-effect of enabling `sqlx`.

**Full Changelog**: https://github.com/SeaQL/sea-orm/compare/0.8.0...0.9.0

## sea-orm-migration 0.8.3

* Removed `async-std` from dependency https://github.com/SeaQL/sea-orm/pull/758

## 0.8.0 - 2022-05-10

### New Features
* [sea-orm-cli] `sea migrate generate` to generate a new, empty migration file https://github.com/SeaQL/sea-orm/pull/656

### Enhancements
* Add `max_connections` option to CLI https://github.com/SeaQL/sea-orm/pull/670
* Derive `Eq`, `Clone` for `DbErr` https://github.com/SeaQL/sea-orm/pull/677
* Add `is_changed` to `ActiveModelTrait` https://github.com/SeaQL/sea-orm/pull/683

### Bug Fixes
* Fix `DerivePrimaryKey` with custom primary key column name https://github.com/SeaQL/sea-orm/pull/694
* Fix `DeriveEntityModel` macros override column name https://github.com/SeaQL/sea-orm/pull/695
* Fix Insert with no value supplied using `DEFAULT` https://github.com/SeaQL/sea-orm/pull/589

### Breaking Changes
* Migration utilities are moved from sea-schema to sea-orm repo, under a new sub-crate `sea-orm-migration`. `sea_schema::migration::prelude` should be replaced by `sea_orm_migration::prelude` in all migration files

### Upgrades
* Upgrade `sea-query` to 0.24.x, `sea-schema` to 0.8.x
* Upgrade example to Actix Web 4, Actix Web 3 remains https://github.com/SeaQL/sea-orm/pull/638
* Added Tonic gRPC example https://github.com/SeaQL/sea-orm/pull/659
* Upgrade GraphQL example to use axum 0.5.x
* Upgrade axum example to 0.5.x

### Fixed Issues
* Failed to insert row with only default values https://github.com/SeaQL/sea-orm/issues/420
* Reduce database connections to 1 during codegen https://github.com/SeaQL/sea-orm/issues/511
* Column names with single letters separated by underscores are concatenated https://github.com/SeaQL/sea-orm/issues/630
* Update Actix Web examples https://github.com/SeaQL/sea-orm/issues/639
* Lower function missing https://github.com/SeaQL/sea-orm/issues/672
* is_changed on active_model https://github.com/SeaQL/sea-orm/issues/674
* Failing find_with_related with column_name attribute https://github.com/SeaQL/sea-orm/issues/693

**Full Changelog**: https://github.com/SeaQL/sea-orm/compare/0.7.1...0.8.0

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

### Breaking Changes
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
* how to update dynamically from json value https://github.com/SeaQL/sea-orm/issues/346
* Make `DatabaseConnection` `Clone` with the default features enabled https://github.com/SeaQL/sea-orm/issues/438
* Updating multiple fields in a Model by passing a reference https://github.com/SeaQL/sea-orm/issues/460
* SeaORM CLI not adding serde derives to Enums https://github.com/SeaQL/sea-orm/issues/461
* sea-orm-cli generates wrong data type for nullable blob https://github.com/SeaQL/sea-orm/issues/490
* Support the time crate in addition (instead of?) chrono https://github.com/SeaQL/sea-orm/issues/499
* PaginatorTrait for SelectorRaw https://github.com/SeaQL/sea-orm/issues/500
* sea_orm::DatabaseConnection should implement `Clone` by default https://github.com/SeaQL/sea-orm/issues/517
* How do you seed data in migrations using ActiveModels? https://github.com/SeaQL/sea-orm/issues/522
* Datetime fields are not serialized by `.into_json()` on queries https://github.com/SeaQL/sea-orm/issues/530
* Update / Delete by id https://github.com/SeaQL/sea-orm/issues/552
* `#[sea_orm(indexed)]` only works for MySQL https://github.com/SeaQL/sea-orm/issues/554
* `sea-orm-cli generate --with-serde` does not work on Postgresql custom type https://github.com/SeaQL/sea-orm/issues/581
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
* Why insert, update, etc return an ActiveModel instead of Model? https://github.com/SeaQL/sea-orm/issues/289
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
* PR without clippy warnings in file changed tab by @billy1624 in https://github.com/SeaQL/sea-orm/pull/401
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
* How to define the rust type of TIMESTAMP? https://github.com/SeaQL/sea-orm/issues/344
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
* Add `PaginatorTrait` and `CountTrait` for more constraints by @YoshieraHuang in https://github.com/SeaQL/sea-orm/pull/306
* Continue `PaginatorTrait` by @billy1624 in https://github.com/SeaQL/sea-orm/pull/307
* Refactor `Schema` by @billy1624 in https://github.com/SeaQL/sea-orm/pull/309
* Detailed connection errors by @billy1624 in https://github.com/SeaQL/sea-orm/pull/312
* Suppress `ouroboros` missing docs warnings by @billy1624 in https://github.com/SeaQL/sea-orm/pull/288
* `with-json` feature requires `chrono/serde` by @billy1624 in https://github.com/SeaQL/sea-orm/pull/320
* Pass the argument `entity.table_ref()` instead of just `entity`. by @josh-codes in https://github.com/SeaQL/sea-orm/pull/318
* Unknown types could be a newtypes instead of `ActiveEnum` by @billy1624 in https://github.com/SeaQL/sea-orm/pull/324
* Returning by @billy1624 in https://github.com/SeaQL/sea-orm/pull/292

### Breaking Changes
* Refactor `paginate()` & `count()` utilities into `PaginatorTrait`. You can use the paginator as usual but you might need to import `PaginatorTrait` manually when upgrading from the previous version.
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
* The helper struct `Schema` converting `EntityTrait` into different `sea-query` statements now has to be initialized with `DbBackend`.
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
* Align case transforms across derive macros https://github.com/SeaQL/sea-orm/issues/262
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
- [[#208]] Support fetching T, (T, U), (T, U, P) etc
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
- [[#175]] Impl `TryGetableMany` for different types of generics
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
