# SeaORM Project Guidelines

This project uses **SeaORM 2.0**. AI models likely have SeaORM 1.0 in their training data -- some patterns have changed. Always follow the 2.0 patterns shown below.

## Quick Reference Links

- [Walk-through of SeaORM 2.0](https://www.sea-ql.org/blog/2025-12-05-sea-orm-2.0/)
- [Migration Guide (1.0 to 2.0)](https://www.sea-ql.org/blog/2026-01-12-sea-orm-2.0/)
- [New Entity Format](https://www.sea-ql.org/blog/2025-10-20-sea-orm-2.0/)
- [Strongly-Typed Column](https://www.sea-ql.org/blog/2025-11-11-sea-orm-2.0/)
- [Nested ActiveModel](https://www.sea-ql.org/blog/2025-11-25-sea-orm-2.0/)
- [Entity First Workflow](https://www.sea-ql.org/blog/2025-10-30-sea-orm-2.0/)

## Entity Definition (2.0 Format)

In SeaORM 2.0, entities use `#[sea_orm::model]` with relations defined directly on the `Model` struct. This replaces the 1.0 pattern of separate `Relation` enums and `Related` trait impls.

```rust
mod user {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "user")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        #[sea_orm(unique)]
        pub email: String,
        #[sea_orm(has_one)]
        pub profile: HasOne<super::profile::Entity>,
        #[sea_orm(has_many)]
        pub posts: HasMany<super::post::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}
```

### Relation Attributes

```rust
// Has-One
#[sea_orm(has_one)]
pub profile: HasOne<super::profile::Entity>,

// Has-Many
#[sea_orm(has_many)]
pub posts: HasMany<super::post::Entity>,

// Belongs-To (explicit foreign key mapping)
#[sea_orm(belongs_to, from = "user_id", to = "id")]
pub user: HasOne<super::user::Entity>,

// Many-to-Many via junction table
#[sea_orm(has_many, via = "post_tag")]
pub tags: HasMany<super::tag::Entity>,

// Self-referential
#[sea_orm(self_ref, via = "user_follower", from = "User", to = "Follower")]
pub followers: HasMany<Entity>,
```

### Junction Table (Composite Primary Key)

```rust
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "post_tag")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub post_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub tag_id: i32,
    #[sea_orm(belongs_to, from = "post_id", to = "id")]
    pub post: Option<super::post::Entity>,
    #[sea_orm(belongs_to, from = "tag_id", to = "id")]
    pub tag: Option<super::tag::Entity>,
}
```

## Strongly-Typed Columns (2.0)

Use `COLUMN` constant with typed fields instead of the untyped `Column` enum:

```rust
// 2.0 (preferred) -- compile-time type safety
user::Entity::find().filter(user::COLUMN.name.contains("Bob"))

// 1.0 (outdated) -- still works but prefer COLUMN
user::Entity::find().filter(user::Column::Name.contains("Bob"))
```

## ActiveModel Builder Pattern (2.0)

```rust
// Create with nested relations
let bob = user::ActiveModel::builder()
    .set_name("Bob")
    .set_email("bob@sea-ql.org")
    .set_profile(profile::ActiveModel::builder().set_picture("Tennis"))
    .insert(db)
    .await?;

// Add has-many children
let mut bob = bob.into_active_model();
bob.posts.push(
    post::ActiveModel::builder().set_title("My first post")
);
bob.save(db).await?;

// Many-to-many
let post = post::ActiveModel::builder()
    .set_title("A sunny day")
    .set_user_id(bob.id)
    .add_tag(existing_tag)
    .add_tag(tag::ActiveModel::builder().set_tag("outdoor"))
    .save(db)
    .await?;
```

## Entity Loader API (2.0)

```rust
// Load with relations in a single query
let bob = user::Entity::load()
    .filter_by_email("bob@sea-ql.org")
    .with(profile::Entity)
    .with(post::Entity)
    .one(db)
    .await?
    .expect("Not found");

// Nested relations (post -> comments)
let user = user::Entity::load()
    .filter_by_id(12)
    .with(profile::Entity)
    .with((post::Entity, comment::Entity))
    .one(db)
    .await?;
```

## Schema Registry (Entity-First Workflow)

```rust
// Auto-create tables from entity definitions (dev/testing)
db.get_schema_registry("my_crate::*")
    .sync(db)
    .await?;
```

## Anti-Patterns -- DO NOT DO THESE

### 1. Do not specify `column_type` on custom wrapper types

When using `DeriveValueType` for custom types, the column type is inferred automatically from the inner type. Adding `column_type` is redundant and incorrect:

```rust
// WRONG -- do not annotate column_type on custom types
#[sea_orm(column_type = "Decimal(Some((10, 4)))")]
pub speed: Speed,

// CORRECT -- SeaORM infers the column type from the DeriveValueType inner type
pub speed: Speed,

#[derive(Clone, Debug, PartialEq, DeriveValueType)]
pub struct Speed(Decimal);
```

### 2. Use `Text` or explicit max length for long strings on MySQL/MSSQL

On MySQL and MSSQL, `String` maps to `VARCHAR(255)` by default. For strings that may exceed 255 characters, use `Text` or specify `StringLen::Max`:

```rust
// WRONG on MySQL/MSSQL -- silently truncates at 255 chars
pub description: String,

// CORRECT -- use column_type for longer strings
#[sea_orm(column_type = "Text")]
pub description: String,

// Also correct -- explicit max length
#[sea_orm(column_type = "String(StringLen::Max)")]
pub event_type: String,
```

Note: Postgre / SQLite uses unbounded string by default, so this is primarily a MySQL/MSSQL concern.

### 3. Missing `ExprTrait` import

Methods like `.eq()`, `.like()`, `.contains()` on `Expr` require the trait import in 2.0:

```rust
use sea_orm::ExprTrait; // required in 2.0

Expr::col((self.entity_name(), *self)).like(s)
```

### 4. Do not use removed or renamed APIs

| 1.0 (removed/renamed) | 2.0 (correct) |
|---|---|
| `.into_condition()` | `.into()` |
| `db.execute(Statement::from_sql_and_values(..))` | `db.execute_raw(Statement::from_sql_and_values(..))` |
| `db.query_all(backend.build(&query))` | `db.query_all(&query)` |
| `Alias::new("col")` for static strings | `Expr::col("col")` directly |
| `insert_many(..).on_empty_do_nothing()` | `insert_many([])` returns `None` safely |

### 5. Do not manually impl traits that `DeriveValueType` now generates

In 2.0, `DeriveValueType` auto-generates `NotU8`, `IntoActiveValue`, and `TryFromU64`. Remove manual implementations to avoid conflicts.

### 6. PostgreSQL: `serial` is no longer the default

Auto-increment columns now use `GENERATED BY DEFAULT AS IDENTITY`. If you need legacy `serial` behavior, use feature flag `option-postgres-use-serial` or `.custom("serial")`.

### 7. SQLite: integer type mapping changed

Both `Integer` and `BigInteger` map to `integer` in 2.0. The entity generator produces `i64` by default. Override with `sea-orm-cli --big-integer-type=i32` if needed.
