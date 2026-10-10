use crate::{
    ActiveEnum, ColumnTrait, ColumnType, ColumnTypeTrait, DbBackend, EntityTrait, IdenStatic,
    Iterable, PrimaryKeyArity, PrimaryKeyToColumn, PrimaryKeyTrait, RelationTrait, Schema,
};
use sea_query::{
    ColumnDef, DynIden, Iden, Index, IndexCreateStatement, SeaRc, TableCreateStatement, TableName,
    TableRef,
    extension::postgres::{Type, TypeCreateStatement},
};
use std::collections::BTreeMap;

impl Schema {
    /// Creates Postgres enums from an ActiveEnum. See [`TypeCreateStatement`] for more details.
    /// Returns None if not Postgres.
    pub fn create_enum_from_active_enum<A>(&self) -> Option<TypeCreateStatement>
    where
        A: ActiveEnum,
    {
        create_enum_from_active_enum::<A>(self.backend)
    }

    /// Creates Postgres enums from an Entity. See [`TypeCreateStatement`] for more details.
    /// Returns empty vec if not Postgres.
    ///
    /// # Panics
    ///
    /// Panics if a PostgreSQL [`ColumnTrait::column_type_override`] changes a
    /// native enum or enum array, which would disagree with runtime casts.
    pub fn create_enum_from_entity<E>(&self, entity: E) -> Vec<TypeCreateStatement>
    where
        E: EntityTrait,
    {
        create_enum_from_entity(entity, self.backend)
    }

    /// Creates a table from an Entity. See [TableCreateStatement] for more details.
    ///
    /// # Panics
    ///
    /// Panics if a PostgreSQL [`ColumnTrait::column_type_override`] changes a
    /// native enum or enum array, which would disagree with runtime casts.
    pub fn create_table_from_entity<E>(&self, entity: E) -> TableCreateStatement
    where
        E: EntityTrait,
    {
        create_table_from_entity(entity, self.backend)
    }

    #[doc(hidden)]
    pub fn create_table_with_index_from_entity<E>(&self, entity: E) -> TableCreateStatement
    where
        E: EntityTrait,
    {
        let mut table = create_table_from_entity(entity, self.backend);
        for mut index in create_index_from_entity(entity, self.backend) {
            table.index(&mut index);
        }
        table
    }

    /// Creates the indexes from an Entity, returning an empty Vec if there are none
    /// to create. See [IndexCreateStatement] for more details
    pub fn create_index_from_entity<E>(&self, entity: E) -> Vec<IndexCreateStatement>
    where
        E: EntityTrait,
    {
        create_index_from_entity(entity, self.backend)
    }

    /// Creates a column definition for example to update a table.
    ///
    /// # Panics
    ///
    /// Panics if a PostgreSQL [`ColumnTrait::column_type_override`] changes a
    /// native enum or enum array, which would disagree with runtime casts.
    ///
    /// ```
    /// use sea_orm::sea_query::TableAlterStatement;
    /// use sea_orm::{DbBackend, Schema, Statement};
    ///
    /// mod post {
    ///     use sea_orm::entity::prelude::*;
    ///
    ///     #[sea_orm::model]
    ///     #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    ///     #[sea_orm(table_name = "posts")]
    ///     pub struct Model {
    ///         #[sea_orm(primary_key)]
    ///         pub id: u32,
    ///         pub title: String,
    ///     }
    ///
    ///     impl ActiveModelBehavior for ActiveModel {}
    /// }
    ///
    /// let schema = Schema::new(DbBackend::MySql);
    ///
    /// let alter_table: Statement = DbBackend::MySql.build(
    ///     TableAlterStatement::new()
    ///         .table(post::Entity)
    ///         .add_column(&mut schema.get_column_def::<post::Entity>(post::Column::Title)),
    /// );
    /// assert_eq!(
    ///     alter_table.to_string(),
    ///     "ALTER TABLE `posts` ADD COLUMN `title` varchar(255) NOT NULL"
    /// );
    /// ```
    pub fn get_column_def<E>(&self, column: E::Column) -> ColumnDef
    where
        E: EntityTrait,
    {
        column_def_from_entity_column::<E>(column, self.backend)
    }
}

pub(crate) fn create_enum_from_active_enum<A>(backend: DbBackend) -> Option<TypeCreateStatement>
where
    A: ActiveEnum,
{
    if matches!(backend, DbBackend::MySql | DbBackend::Sqlite) {
        return None;
    }
    let col_def = A::db_type();
    let col_type = col_def.get_column_type();
    create_enum_from_column_type(col_type)
}

pub(crate) fn create_enum_from_column_type(col_type: &ColumnType) -> Option<TypeCreateStatement> {
    let (name, values) = match col_type {
        ColumnType::Enum { name, variants } => (name.clone(), variants.clone()),
        _ => return None,
    };
    Some(Type::create().as_enum(name).values(values).to_owned())
}

#[allow(clippy::needless_borrow)]
pub(crate) fn create_enum_from_entity<E>(_: E, backend: DbBackend) -> Vec<TypeCreateStatement>
where
    E: EntityTrait,
{
    if matches!(backend, DbBackend::MySql | DbBackend::Sqlite) {
        return Vec::new();
    }
    let mut vec = Vec::new();
    for col in E::Column::iter() {
        let col_def = column_def_for_backend(col, backend);
        let col_type = col_def.get_column_type();
        if let Some(stmt) = create_enum_from_column_type(&col_type) {
            vec.push(stmt);
        }
    }
    vec
}

pub(crate) fn create_index_from_entity<E>(
    entity: E,
    backend: DbBackend,
) -> Vec<IndexCreateStatement>
where
    E: EntityTrait,
{
    let mut indexes = Vec::new();
    let mut unique_keys: BTreeMap<String, Vec<DynIden>> = Default::default();

    for column in E::Column::iter() {
        let column_def = column.def();

        if column_def.indexed && !column_def.unique {
            let stmt = Index::create()
                .name(format!("idx-{}-{}", entity.to_string(), column.to_string()))
                .table(index_table_ref(entity.table_ref(), backend))
                .col(column)
                .take();
            indexes.push(stmt);
        }

        if let Some(key) = column_def.unique_key {
            unique_keys.entry(key).or_default().push(SeaRc::new(column));
        }
    }

    for (key, cols) in unique_keys {
        let mut stmt = Index::create()
            .name(format!("idx-{}-{}", entity.to_string(), key))
            .table(index_table_ref(entity.table_ref(), backend))
            .unique()
            .take();
        for col in cols {
            stmt.col(col);
        }
        indexes.push(stmt);
    }

    indexes
}

/// Build the table reference used for a generated index.
///
/// PostgreSQL accepts a schema-qualified index target
/// (`CREATE INDEX ... ON "schema"."table"`), so a `schema_name` qualifier is
/// preserved. SeaQuery's MySQL and SQLite index builders accept only a bare
/// table name and panic on a qualified one; their generated index is implicitly
/// scoped to the table's database/schema anyway, so the qualifier is stripped.
pub(crate) fn index_table_ref(table_ref: TableRef, backend: DbBackend) -> TableRef {
    match backend {
        DbBackend::Postgres => table_ref,
        DbBackend::MySql | DbBackend::Sqlite => match table_ref {
            TableRef::Table(TableName(Some(_), table), alias) => {
                TableRef::Table(TableName(None, table), alias)
            }
            other => other,
        },
    }
}

pub(crate) fn create_table_from_entity<E>(entity: E, backend: DbBackend) -> TableCreateStatement
where
    E: EntityTrait,
{
    let mut stmt = TableCreateStatement::new();

    if let Some(comment) = entity.comment() {
        stmt.comment(comment);
    }

    for column in E::Column::iter() {
        let mut column_def = column_def_from_entity_column::<E>(column, backend);
        stmt.col(&mut column_def);
    }

    if <<E::PrimaryKey as PrimaryKeyTrait>::ValueType as PrimaryKeyArity>::ARITY > 1 {
        let mut idx_pk = Index::create();
        for primary_key in E::PrimaryKey::iter() {
            idx_pk.col(primary_key);
        }
        stmt.primary_key(idx_pk.name(format!("pk-{}", entity.to_string())).primary());
    }

    for relation in E::Relation::iter() {
        let relation = relation.def();
        if relation.is_owner || relation.skip_fk {
            continue;
        }
        stmt.foreign_key(&mut relation.into());
    }

    stmt.table(entity.table_ref()).take()
}

fn column_def_from_entity_column<E>(column: E::Column, backend: DbBackend) -> ColumnDef
where
    E: EntityTrait,
{
    let orm_column_def = column_def_for_backend(column, backend);
    // Resolve the physical type before applying the backend's enum rendering.
    let types = match &orm_column_def.col_type {
        ColumnType::Enum { name, variants } => match backend {
            DbBackend::MySql => {
                let variants: Vec<String> = variants.iter().map(|v| v.to_string()).collect();
                ColumnType::custom(format!("ENUM('{}')", variants.join("', '")))
            }
            DbBackend::Postgres => ColumnType::Custom(name.clone()),
            DbBackend::Sqlite => orm_column_def.col_type,
        },
        _ => orm_column_def.col_type,
    };
    let mut column_def = ColumnDef::new_with_type(column, types);
    if !orm_column_def.null {
        column_def.not_null();
    }
    if orm_column_def.unique {
        column_def.unique_key();
    }
    if let Some(default) = orm_column_def.default {
        column_def.default(default);
    }
    if let Some(comment) = &orm_column_def.comment {
        column_def.comment(comment);
    }
    if let Some(extra) = &orm_column_def.extra {
        column_def.extra(extra);
    }
    match (&orm_column_def.renamed_from, &orm_column_def.comment) {
        (Some(renamed_from), Some(comment)) => {
            column_def.comment(format!("{comment}; renamed_from \"{renamed_from}\""));
        }
        (Some(renamed_from), None) => {
            column_def.comment(format!("renamed_from \"{renamed_from}\""));
        }
        (None, _) => {}
    }
    for primary_key in E::PrimaryKey::iter() {
        if column.as_str() == primary_key.into_column().as_str() {
            if E::PrimaryKey::auto_increment() {
                column_def.auto_increment();
            }
            if <<E::PrimaryKey as PrimaryKeyTrait>::ValueType as PrimaryKeyArity>::ARITY == 1 {
                column_def.primary_key();
            }
        }
    }
    column_def
}

fn column_def_for_backend<C: ColumnTrait>(column: C, backend: DbBackend) -> crate::ColumnDef {
    let mut def = column.def();
    if let Some(col_type) = column.column_type_override(backend) {
        // PostgreSQL query casts and enum values retain their logical type name.
        // Changing a native enum here would make schema and queries disagree.
        if backend == DbBackend::Postgres
            && (def.col_type.get_enum_name().is_some() || col_type.get_enum_name().is_some())
        {
            assert_eq!(
                def.col_type,
                col_type,
                "PostgreSQL column type override for `{}` cannot change a native enum or enum array",
                column.as_str(),
            );
        }
        def.col_type = col_type;
    }
    def
}

#[cfg(test)]
mod tests {
    use crate::{DbBackend, EntityName, Schema, sea_query::*, tests_cfg::*};
    use pretty_assertions::assert_eq;

    mod per_backend_column_type {
        use crate as sea_orm;
        use crate::entity::prelude::*;

        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "payload_item")]
        pub struct Model {
            #[sea_orm(primary_key, auto_increment = false)]
            pub id: i32,
            #[sea_orm(column_type = "Json", column_type_postgres = "JsonBinary")]
            pub payload: Json,
            #[sea_orm(
                column_type = "TinyInteger",
                column_type_mysql = "SmallInteger",
                unique,
                default_value = 7
            )]
            pub level: Option<i16>,
            #[sea_orm(column_type_sqlite = "Text", indexed, default_value = "draft")]
            pub label: String,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    #[test]
    fn test_create_table_from_entity_per_backend_column_type() {
        use per_backend_column_type as payload_item;

        for backend in [DbBackend::Postgres, DbBackend::MySql, DbBackend::Sqlite] {
            let schema = Schema::new(backend);
            // The override wins on its backend; other types retain their default rendering.
            let payload_type = match backend {
                DbBackend::Postgres => ColumnType::JsonBinary,
                _ => ColumnType::Json,
            };
            let level_type = match backend {
                DbBackend::MySql => ColumnType::SmallInteger,
                _ => ColumnType::TinyInteger,
            };
            let label_type = match backend {
                DbBackend::Sqlite => ColumnType::Text,
                _ => ColumnType::String(StringLen::None),
            };
            let expected = Table::create()
                .table(payload_item::Entity)
                .col(
                    ColumnDef::new(payload_item::Column::Id)
                        .integer()
                        .not_null()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new_with_type(payload_item::Column::Payload, payload_type)
                        .not_null(),
                )
                .col(
                    ColumnDef::new_with_type(payload_item::Column::Level, level_type)
                        .unique_key()
                        .default(7),
                )
                .col(
                    ColumnDef::new_with_type(payload_item::Column::Label, label_type)
                        .not_null()
                        .default("draft"),
                )
                .to_owned();
            assert_eq!(
                backend.build(&schema.create_table_from_entity(payload_item::Entity)),
                backend.build(&expected),
            );
            // ALTER TABLE must use the same physical type and preserve constraints/defaults.
            let actual = Table::alter()
                .table(payload_item::Entity)
                .add_column(
                    schema.get_column_def::<payload_item::Entity>(payload_item::Column::Level),
                )
                .to_owned();
            let expected_alter = Table::alter()
                .table(payload_item::Entity)
                .add_column(expected.get_columns()[2].clone())
                .to_owned();
            assert_eq!(backend.build(&actual), backend.build(&expected_alter));
            let expected_index = Index::create()
                .name("idx-payload_item-label")
                .table(payload_item::Entity)
                .col(payload_item::Column::Label)
                .to_owned();
            let indexes = schema.create_index_from_entity(payload_item::Entity);
            assert_eq!(indexes.len(), 1);
            assert_eq!(backend.build(&indexes[0]), backend.build(&expected_index));
        }
    }

    mod lazy_column_type {
        use crate as sea_orm;
        use crate::entity::prelude::*;
        use std::cell::Cell;

        thread_local! {
            pub static CALLS: Cell<[usize; 3]> = const { Cell::new([0; 3]) };
        }

        fn length(backend: usize) -> StringLen {
            CALLS.with(|calls| {
                let mut counts = calls.get();
                counts[backend] += 1;
                calls.set(counts);
            });
            StringLen::N(80)
        }

        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "lazy_item")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            #[sea_orm(
                column_type = "Text",
                column_type_mysql = "String(length(0))",
                column_type_postgres = "String(length(1))",
                column_type_sqlite = "String(length(2))"
            )]
            pub value: String,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    #[test]
    fn test_column_type_overrides_are_only_evaluated_for_schema() {
        use crate::{ColumnTrait, EntityTrait, QueryFilter, QueryTrait, Set};
        use lazy_column_type::{ActiveModel, CALLS, Column, Entity};

        CALLS.with(|calls| calls.set([0; 3]));
        assert_eq!(Column::Value.def().get_column_type(), &ColumnType::Text);
        for backend in [DbBackend::MySql, DbBackend::Postgres, DbBackend::Sqlite] {
            let model = ActiveModel {
                id: Set(1),
                value: Set("value".into()),
            };
            Entity::find()
                .filter(Column::Value.eq("value"))
                .build(backend);
            Entity::insert(model.clone()).build(backend);
            Entity::update(model).validate().unwrap().build(backend);
            CALLS.with(|calls| assert_eq!(calls.get(), [0; 3]));
            assert_eq!(Column::Id.column_type_override(backend), None);
        }
        assert_eq!(
            Entity::find()
                .filter(Column::Value.eq("value"))
                .build(DbBackend::Postgres)
                .to_string(),
            r#"SELECT "lazy_item"."id", "lazy_item"."value" FROM "lazy_item" WHERE "lazy_item"."value" = 'value'"#,
        );
        for (index, backend) in [DbBackend::MySql, DbBackend::Postgres, DbBackend::Sqlite]
            .into_iter()
            .enumerate()
        {
            CALLS.with(|calls| calls.set([0; 3]));
            Schema::new(backend).create_table_from_entity(Entity);
            let mut expected = [0; 3];
            expected[index] = 1;
            CALLS.with(|calls| assert_eq!(calls.get(), expected));
        }
    }

    mod enum_overrides {
        use crate as sea_orm;
        use crate::entity::prelude::*;

        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "enum_item")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            #[sea_orm(
                column_type = "Text",
                column_type_postgres = "Enum { name: \"status\".into(), variants: vec![\"ready\".into()] }"
            )]
            pub promoted: String,
            #[sea_orm(
                column_type = "Enum { name: \"status\".into(), variants: vec![\"ready\".into()] }",
                column_type_postgres = "Text"
            )]
            pub demoted: String,
            #[sea_orm(
                column_type = "Enum { name: \"status\".into(), variants: vec![\"ready\".into()] }",
                column_type_postgres = "Enum { name: \"other_status\".into(), variants: vec![\"ready\".into()] }"
            )]
            pub renamed: String,
            #[sea_orm(
                column_type = "Enum { name: \"status\".into(), variants: vec![\"ready\".into()] }",
                column_type_postgres = "Enum { name: \"status\".into(), variants: vec![\"other\".into()] }"
            )]
            pub changed_variants: String,
            #[sea_orm(
                column_type = "Array(ColumnType::Enum { name: \"status\".into(), variants: vec![\"ready\".into()] }.into())",
                column_type_postgres = "Array(ColumnType::Text.into())"
            )]
            pub array_demoted: String,
            #[sea_orm(
                column_type = "Enum { name: \"status\".into(), variants: vec![\"ready\".into()] }",
                column_type_postgres = "Array(ColumnType::Enum { name: \"status\".into(), variants: vec![\"ready\".into()] }.into())"
            )]
            pub array_promoted: String,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    #[test]
    fn test_postgres_rejects_incompatible_enum_overrides() {
        use enum_overrides::{Column, Entity};

        let schema = Schema::new(DbBackend::Postgres);
        for column in [
            Column::Promoted,
            Column::Demoted,
            Column::Renamed,
            Column::ChangedVariants,
            Column::ArrayDemoted,
            Column::ArrayPromoted,
        ] {
            let error = std::panic::catch_unwind(|| schema.get_column_def::<Entity>(column))
                .expect_err("an incompatible native enum override must be rejected");
            let message = error.downcast_ref::<String>().unwrap();
            assert!(message.contains("cannot change a native enum or enum array"));
        }
        assert!(std::panic::catch_unwind(|| schema.create_table_from_entity(Entity)).is_err());
        assert!(std::panic::catch_unwind(|| schema.create_enum_from_entity(Entity)).is_err());
        assert!(
            std::panic::catch_unwind(|| crate::EntitySchemaInfo::new(Entity, &schema)).is_err()
        );
        // A PostgreSQL-only override must not affect schema generation for other backends.
        assert_eq!(
            Schema::new(DbBackend::MySql)
                .get_column_def::<Entity>(Column::Promoted)
                .get_column_type(),
            Some(&ColumnType::Text)
        );
    }

    mod native_enum {
        use crate as sea_orm;
        use crate::entity::prelude::*;

        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "native_item")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            #[sea_orm(
                column_type = "Enum { name: \"status\".into(), variants: vec![\"ready\".into()] }",
                column_type_postgres = "Enum { name: \"status\".into(), variants: vec![\"ready\".into()] }",
                column_type_sqlite = "Text"
            )]
            pub status: String,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    #[test]
    fn test_native_enum_override_keeps_schema_and_casts_consistent() {
        use crate::{ColumnTrait, EntityTrait, QueryFilter, QueryTrait};
        use native_enum::{Column, Entity};

        let backend = DbBackend::Postgres;
        let schema = Schema::new(backend);
        let enums = schema.create_enum_from_entity(Entity);
        assert_eq!(enums.len(), 1);
        assert_eq!(
            backend.build(&enums[0]).to_string(),
            r#"CREATE TYPE "status" AS ENUM ('ready')"#
        );
        assert_eq!(
            schema
                .get_column_def::<Entity>(Column::Status)
                .get_column_type(),
            Some(&ColumnType::custom("status"))
        );
        assert_eq!(
            Entity::find()
                .filter(Column::Status.eq("ready"))
                .build(backend)
                .to_string(),
            r#"SELECT "native_item"."id", CAST("native_item"."status" AS "text") FROM "native_item" WHERE "native_item"."status" = (CAST('ready' AS "status"))"#
        );
        // MySQL keeps its inline enum, while SQLite can use text without PostgreSQL casts.
        assert!(
            DbBackend::MySql
                .build(&Schema::new(DbBackend::MySql).create_table_from_entity(Entity))
                .to_string()
                .contains("ENUM('ready')")
        );
        assert_eq!(
            Schema::new(DbBackend::Sqlite)
                .get_column_def::<Entity>(Column::Status)
                .get_column_type(),
            Some(&ColumnType::Text)
        );
    }

    mod custom_schema_indexes {
        use crate as sea_orm;
        use crate::entity::prelude::*;

        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(schema_name = "sys", table_name = "app_user")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            #[sea_orm(indexed)]
            pub email: String,
            #[sea_orm(unique_key = "tenant_name")]
            pub tenant_id: i32,
            #[sea_orm(unique_key = "tenant_name")]
            pub name: String,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    #[test]
    fn test_create_table_from_entity_table_ref() {
        for builder in [DbBackend::MySql, DbBackend::Postgres, DbBackend::Sqlite] {
            let schema = Schema::new(builder);
            assert_eq!(
                builder.build(&schema.create_table_from_entity(CakeFillingPrice)),
                builder.build(
                    &get_cake_filling_price_stmt()
                        .table(CakeFillingPrice.table_ref())
                        .to_owned()
                )
            );
        }
    }

    fn get_cake_filling_price_stmt() -> TableCreateStatement {
        Table::create()
            .col(
                ColumnDef::new(cake_filling_price::Column::CakeId)
                    .integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(cake_filling_price::Column::FillingId)
                    .integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(cake_filling_price::Column::Price)
                    .decimal()
                    .not_null()
                    .extra("CHECK (price > 0)"),
            )
            .primary_key(
                Index::create()
                    .name("pk-cake_filling_price")
                    .col(cake_filling_price::Column::CakeId)
                    .col(cake_filling_price::Column::FillingId)
                    .primary(),
            )
            .foreign_key(
                ForeignKeyCreateStatement::new()
                    .name("fk-cake_filling_price-cake_id-filling_id")
                    .from_tbl(CakeFillingPrice)
                    .from_col(cake_filling_price::Column::CakeId)
                    .from_col(cake_filling_price::Column::FillingId)
                    .to_tbl(CakeFilling)
                    .to_col(cake_filling::Column::CakeId)
                    .to_col(cake_filling::Column::FillingId),
            )
            .to_owned()
    }

    #[test]
    fn test_create_index_from_entity_table_ref() {
        for builder in [DbBackend::MySql, DbBackend::Postgres, DbBackend::Sqlite] {
            let schema = Schema::new(builder);

            assert_eq!(
                builder.build(&schema.create_table_from_entity(indexes::Entity)),
                builder.build(
                    &get_indexes_table_stmt()
                        .table(indexes::Entity.table_ref())
                        .to_owned()
                )
            );

            let stmts = schema.create_index_from_entity(indexes::Entity);
            assert_eq!(stmts.len(), 2);

            let index_table = match builder {
                DbBackend::Postgres => indexes::Entity.table_ref(),
                DbBackend::MySql | DbBackend::Sqlite => indexes::Entity.into_table_ref(),
            };
            let idx: IndexCreateStatement = Index::create()
                .name("idx-indexes-index1_attr")
                .table(index_table)
                .col(indexes::Column::Index1Attr)
                .to_owned();
            assert_eq!(builder.build(&stmts[0]), builder.build(&idx));

            let index_table = match builder {
                DbBackend::Postgres => indexes::Entity.table_ref(),
                DbBackend::MySql | DbBackend::Sqlite => indexes::Entity.into_table_ref(),
            };
            let idx: IndexCreateStatement = Index::create()
                .name("idx-indexes-my_unique")
                .table(index_table)
                .col(indexes::Column::UniqueKeyA)
                .col(indexes::Column::UniqueKeyB)
                .unique()
                .take();
            assert_eq!(builder.build(&stmts[1]), builder.build(&idx));
        }
    }

    #[test]
    fn test_create_index_from_entity_non_default_schema_table_ref() {
        let builder = DbBackend::Postgres;
        let schema = Schema::new(builder);
        let stmts = schema.create_index_from_entity(custom_schema_indexes::Entity);
        assert_eq!(stmts.len(), 2);

        let idx: IndexCreateStatement = Index::create()
            .name("idx-app_user-email")
            .table(custom_schema_indexes::Entity.table_ref())
            .col(custom_schema_indexes::Column::Email)
            .to_owned();
        assert_eq!(builder.build(&stmts[0]), builder.build(&idx));

        let idx: IndexCreateStatement = Index::create()
            .name("idx-app_user-tenant_name")
            .table(custom_schema_indexes::Entity.table_ref())
            .col(custom_schema_indexes::Column::TenantId)
            .col(custom_schema_indexes::Column::Name)
            .unique()
            .take();
        assert_eq!(builder.build(&stmts[1]), builder.build(&idx));

        // The generated DDL targets the schema-qualified table.
        assert!(builder.build(&stmts[0]).sql.contains(r#""sys"."app_user""#));
    }

    // Regression guard for the SeaQuery MySQL/SQLite index builders, which panic
    // on a schema-qualified table reference. `create_index_from_entity` must
    // strip the `schema_name` qualifier on those backends, so generation neither
    // panics nor emits a qualified target. See `index_table_ref`.
    #[test]
    fn test_create_index_from_entity_non_default_schema_strips_schema_on_mysql_sqlite() {
        for builder in [DbBackend::MySql, DbBackend::Sqlite] {
            let schema = Schema::new(builder);
            // Must not panic for a `schema_name` entity on MySQL/SQLite.
            let stmts = schema.create_index_from_entity(custom_schema_indexes::Entity);
            assert_eq!(stmts.len(), 2);

            for stmt in &stmts {
                let sql = builder.build(stmt).sql;
                assert!(
                    sql.contains("app_user"),
                    "{builder:?} index should target the table: {sql}"
                );
                assert!(
                    !sql.contains("sys"),
                    "{builder:?} index should not be schema-qualified: {sql}"
                );
            }
        }
    }

    fn get_indexes_table_stmt() -> TableCreateStatement {
        Table::create()
            .col(
                ColumnDef::new(indexes::Column::IndexesId)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(indexes::Column::UniqueAttr)
                    .integer()
                    .not_null()
                    .unique_key(),
            )
            .col(
                ColumnDef::new(indexes::Column::Index1Attr)
                    .integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(indexes::Column::Index2Attr)
                    .integer()
                    .not_null()
                    .unique_key(),
            )
            .col(
                ColumnDef::new(indexes::Column::UniqueKeyA)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(indexes::Column::UniqueKeyB)
                    .string()
                    .not_null(),
            )
            .to_owned()
    }
}
