use crate::{
    ActiveEnum, ColumnTrait, ColumnType, DbBackend, EntityTrait, IdenStatic, Iterable,
    PrimaryKeyArity, PrimaryKeyToColumn, PrimaryKeyTrait, RelationTrait, Schema,
};
use sea_query::{
    ColumnDef, DynIden, Iden, Index, IndexCreateStatement, SeaRc, TableCreateStatement,
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
    pub fn create_enum_from_entity<E>(&self, entity: E) -> Vec<TypeCreateStatement>
    where
        E: EntityTrait,
    {
        create_enum_from_entity(entity, self.backend)
    }

    /// Creates a table from an Entity. See [TableCreateStatement] for more details.
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
        let col_def = col.def();
        let col_type = col_def.get_column_type();
        if !matches!(col_type, ColumnType::Enum { .. }) {
            continue;
        }
        if let Some(stmt) = create_enum_from_column_type(&col_type) {
            vec.push(stmt);
        }
    }
    vec
}

pub(crate) fn create_index_from_entity<E>(
    entity: E,
    _backend: DbBackend,
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
                .table(entity)
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
            .table(entity)
            .unique()
            .take();
        for col in cols {
            stmt.col(col);
        }
        indexes.push(stmt);
    }

    indexes
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
    let orm_column_def = column.def();
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

#[cfg(test)]
mod tests {
    use crate::{DbBackend, EntityName, Schema, sea_query::*, tests_cfg::*};
    use pretty_assertions::assert_eq;

    /// Postgres native enum (db_type = "Enum") — should produce CREATE TYPE on
    /// Postgres, nothing on MySQL/SQLite.
    #[test]
    fn test_create_enum_native_postgres() {
        let schema_pg = Schema::new(DbBackend::Postgres);
        let enums = schema_pg.create_enum_from_entity(lunch_set::Entity);
        assert_eq!(
            enums.len(),
            1,
            "Postgres should produce one CREATE TYPE for the Tea enum"
        );
        let sql = DbBackend::Postgres.build(&enums[0]).to_string();
        assert!(
            sql.contains("CREATE TYPE"),
            "should be a CREATE TYPE statement: {sql}"
        );
        assert!(
            sql.contains("tea"),
            "should reference the enum name 'tea': {sql}"
        );

        // MySQL/SQLite: no enum type statements
        for backend in [DbBackend::MySql, DbBackend::Sqlite] {
            let schema = Schema::new(backend);
            let enums = schema.create_enum_from_entity(lunch_set::Entity);
            assert!(
                enums.is_empty(),
                "{backend:?} should not produce enum type statements"
            );
        }
    }

    /// Postgres native enum column: Postgres references the custom type name,
    /// MySQL uses inline ENUM('v1', 'v2').
    #[test]
    fn test_native_enum_column_type_per_backend() {
        let pg_sql = DbBackend::Postgres
            .build(&Schema::new(DbBackend::Postgres).create_table_from_entity(lunch_set::Entity))
            .to_string();
        assert!(
            pg_sql.contains("\"tea\""),
            "Postgres table should reference custom type 'tea': {pg_sql}"
        );

        let mysql_sql = DbBackend::MySql
            .build(&Schema::new(DbBackend::MySql).create_table_from_entity(lunch_set::Entity))
            .to_string();
        assert!(
            mysql_sql.contains("ENUM("),
            "MySQL table should use inline ENUM(...): {mysql_sql}"
        );
    }

    /// String-based enum (db_type = "String(...)") must NOT produce any
    /// CREATE TYPE statements — it's just a regular string column.
    #[test]
    fn test_create_enum_string_based_no_create_type() {
        use crate as sea_orm;
        use crate::entity::prelude::*;

        #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
        #[sea_orm(rs_type = "String", db_type = "String(StringLen::N(1))")]
        pub enum Size {
            #[sea_orm(string_value = "S")]
            Small,
            #[sea_orm(string_value = "L")]
            Large,
        }

        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "shirt")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            pub size: Size,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}
        impl ActiveModelBehavior for ActiveModel {}

        for backend in [DbBackend::MySql, DbBackend::Postgres, DbBackend::Sqlite] {
            let schema = Schema::new(backend);
            let enums = schema.create_enum_from_entity(Entity);
            assert!(
                enums.is_empty(),
                "{backend:?}: String-based enum should not produce CREATE TYPE"
            );

            // Verify the column appears as a string type in the table DDL
            let table_sql = backend
                .build(&schema.create_table_from_entity(Entity))
                .to_string();
            assert!(
                !table_sql.to_uppercase().contains("CREATE TYPE"),
                "{backend:?}: table DDL should not contain CREATE TYPE: {table_sql}"
            );
        }
    }

    /// Integer-based enum (db_type = "Integer") must NOT produce any
    /// CREATE TYPE statements — it's just a regular integer column.
    #[test]
    fn test_create_enum_integer_based_no_create_type() {
        use crate as sea_orm;
        use crate::entity::prelude::*;

        #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
        #[sea_orm(rs_type = "i32", db_type = "Integer")]
        pub enum Priority {
            #[sea_orm(num_value = 0)]
            Low,
            #[sea_orm(num_value = 1)]
            High,
        }

        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "task")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            pub priority: Priority,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}
        impl ActiveModelBehavior for ActiveModel {}

        for backend in [DbBackend::MySql, DbBackend::Postgres, DbBackend::Sqlite] {
            let schema = Schema::new(backend);
            let enums = schema.create_enum_from_entity(Entity);
            assert!(
                enums.is_empty(),
                "{backend:?}: Integer-based enum should not produce CREATE TYPE"
            );
        }
    }

    /// Entity with no enum columns at all — should produce nothing.
    #[test]
    fn test_create_enum_no_enum_columns() {
        for backend in [DbBackend::MySql, DbBackend::Postgres, DbBackend::Sqlite] {
            let schema = Schema::new(backend);
            let enums = schema.create_enum_from_entity(cake::Entity);
            assert!(
                enums.is_empty(),
                "{backend:?}: entity without enum columns should produce no enum statements"
            );
        }
    }

    /// Entity with multiple Postgres enum columns produces one CREATE TYPE per enum.
    #[test]
    fn test_create_enum_multiple_enum_columns() {
        use crate as sea_orm;
        use crate::entity::prelude::*;

        #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
        #[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "color")]
        pub enum Color {
            #[sea_orm(string_value = "red")]
            Red,
            #[sea_orm(string_value = "blue")]
            Blue,
        }

        #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
        #[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "shape")]
        pub enum Shape {
            #[sea_orm(string_value = "circle")]
            Circle,
            #[sea_orm(string_value = "square")]
            Square,
        }

        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "widget")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            pub color: Color,
            pub shape: Shape,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}
        impl ActiveModelBehavior for ActiveModel {}

        let schema = Schema::new(DbBackend::Postgres);
        let enums = schema.create_enum_from_entity(Entity);
        assert_eq!(
            enums.len(),
            2,
            "should produce two CREATE TYPE statements for two enum columns"
        );

        let sqls: Vec<String> = enums
            .iter()
            .map(|e| DbBackend::Postgres.build(e).to_string())
            .collect();
        assert!(
            sqls.iter().any(|s| s.contains("color")),
            "should have CREATE TYPE for 'color': {sqls:?}"
        );
        assert!(
            sqls.iter().any(|s| s.contains("shape")),
            "should have CREATE TYPE for 'shape': {sqls:?}"
        );

        // MySQL: no CREATE TYPE
        let mysql_enums = Schema::new(DbBackend::MySql).create_enum_from_entity(Entity);
        assert!(mysql_enums.is_empty());
    }

    /// Mixed entity: one Postgres native enum, one string enum, one integer enum.
    /// Only the native enum should produce CREATE TYPE on Postgres.
    #[test]
    fn test_create_enum_mixed_column_types() {
        use crate as sea_orm;
        use crate::entity::prelude::*;

        #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
        #[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "mood")]
        pub enum Mood {
            #[sea_orm(string_value = "happy")]
            Happy,
            #[sea_orm(string_value = "sad")]
            Sad,
        }

        #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
        #[sea_orm(rs_type = "String", db_type = "String(StringLen::N(10))")]
        pub enum Tag {
            #[sea_orm(string_value = "work")]
            Work,
            #[sea_orm(string_value = "play")]
            Play,
        }

        #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
        #[sea_orm(rs_type = "i32", db_type = "Integer")]
        pub enum Level {
            #[sea_orm(num_value = 1)]
            One,
            #[sea_orm(num_value = 2)]
            Two,
        }

        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "entry")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            pub mood: Mood,
            pub tag: Tag,
            pub level: Level,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}
        impl ActiveModelBehavior for ActiveModel {}

        // Only the native Postgres enum should produce a CREATE TYPE
        let schema = Schema::new(DbBackend::Postgres);
        let enums = schema.create_enum_from_entity(Entity);
        assert_eq!(
            enums.len(),
            1,
            "only the native Postgres enum (Mood) should produce CREATE TYPE"
        );
        let sql = DbBackend::Postgres.build(&enums[0]).to_string();
        assert!(sql.contains("mood"), "should be the 'mood' enum: {sql}");
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

            let idx: IndexCreateStatement = Index::create()
                .name("idx-indexes-index1_attr")
                .table(indexes::Entity)
                .col(indexes::Column::Index1Attr)
                .to_owned();
            assert_eq!(builder.build(&stmts[0]), builder.build(&idx));

            let idx: IndexCreateStatement = Index::create()
                .name("idx-indexes-my_unique")
                .table(indexes::Entity)
                .col(indexes::Column::UniqueKeyA)
                .col(indexes::Column::UniqueKeyB)
                .unique()
                .take();
            assert_eq!(builder.build(&stmts[1]), builder.build(&idx));
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
