// ---------------------------------------------------------------------------
// Entity definitions — realistic schema with relations, unique constraints,
// foreign keys, and versioned variants for testing discover() scenarios.
// ---------------------------------------------------------------------------

/// `cake` — has a unique name, owns many `fruit`s.
pub mod cake {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "cake")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(unique)]
        pub name: String,
        #[sea_orm(has_many)]
        pub fruits: HasMany<super::fruit::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// `fruit` — belongs to a `cake` via foreign key.
pub mod fruit {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "fruit")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        pub cake_id: i32,
        #[sea_orm(belongs_to, from = "cake_id", to = "id")]
        pub cake: HasOne<super::cake::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// `cake_v1` — initial version of cake without a unique name (for diff tests).
pub mod cake_v1 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "cake")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// `cake_v2` — adds a `description` column and a `category` column.
pub mod cake_v2 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "cake")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        #[sea_orm(column_type = "Text")]
        pub description: String,
        pub category: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// `fruit_v1` — initial fruit without a `weight` column.
pub mod fruit_v1 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "fruit")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        pub cake_id: i32,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// `fruit_v2` — adds a `weight_grams` column (integer) and a `unique` constraint on `name`.
pub mod fruit_v2 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "fruit")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(unique)]
        pub name: String,
        pub cake_id: i32,
        pub weight_grams: i32,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// `cake_renamed` — same schema as `cake_v1` but `name` is renamed to `title` (same String type).
/// Used for testing rename detection heuristic.
pub mod cake_renamed {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "cake")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub title: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// `cake_type_change` — same schema as `cake_v1` but `name` removed and `count` (i32) added.
/// The type differs, so rename detection should NOT trigger.
pub mod cake_type_change {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "cake")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub count: i32,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// `cake_ambiguous` — same schema as `cake_v1` but `name` removed, `title` and `label`
/// both added (same String type). This creates an ambiguous rename scenario.
pub mod cake_ambiguous {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "cake")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub title: String,
        pub label: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

// ---------------------------------------------------------------------------
// EntitySet implementations
// ---------------------------------------------------------------------------

use sea_orm_migration::{EntitySet, SchemaBuilder};

/// Full schema: cake + fruit with FK relation.
pub struct FullSchema;
impl EntitySet for FullSchema {
    fn register(self, builder: SchemaBuilder) -> SchemaBuilder {
        builder
            .register(cake::Entity)
            .register(fruit::Entity)
    }
}

/// Only cake v1 (no unique, no extra columns).
pub struct CakeV1Only;
impl EntitySet for CakeV1Only {
    fn register(self, builder: SchemaBuilder) -> SchemaBuilder {
        builder.register(cake_v1::Entity)
    }
}

/// Cake v2 + fruit v1 (cake gains columns, fruit gains nothing yet).
pub struct CakeV2FruitV1;
impl EntitySet for CakeV2FruitV1 {
    fn register(self, builder: SchemaBuilder) -> SchemaBuilder {
        builder
            .register(cake_v2::Entity)
            .register(fruit_v1::Entity)
    }
}

/// Cake v1 + fruit v2 (fruit gains a column and a unique index).
pub struct CakeV1FruitV2;
impl EntitySet for CakeV1FruitV2 {
    fn register(self, builder: SchemaBuilder) -> SchemaBuilder {
        builder
            .register(cake_v1::Entity)
            .register(fruit_v2::Entity)
    }
}

/// Cake with `name` renamed to `title` (same String type) — for rename detection tests.
pub struct CakeRenamedOnly;
impl EntitySet for CakeRenamedOnly {
    fn register(self, builder: SchemaBuilder) -> SchemaBuilder {
        builder.register(cake_renamed::Entity)
    }
}

/// Cake with `name` removed and `count` (i32) added — type mismatch, no rename.
pub struct CakeTypeChangeOnly;
impl EntitySet for CakeTypeChangeOnly {
    fn register(self, builder: SchemaBuilder) -> SchemaBuilder {
        builder.register(cake_type_change::Entity)
    }
}

/// Cake with `name` removed and both `title` + `label` added — ambiguous rename.
pub struct CakeAmbiguousOnly;
impl EntitySet for CakeAmbiguousOnly {
    fn register(self, builder: SchemaBuilder) -> SchemaBuilder {
        builder.register(cake_ambiguous::Entity)
    }
}
