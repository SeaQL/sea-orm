use crate::{
    unpack_table_ref, ActiveEnum, ColumnTrait, ColumnType, DbBackend, EntityTrait, Identity,
    Iterable, PrimaryKeyToColumn, PrimaryKeyTrait, RelationTrait, Schema,
};
use sea_query::{
    extension::postgres::{Type, TypeCreateStatement},
    Alias, ColumnDef, ForeignKeyCreateStatement, Iden, Index, IndexCreateStatement, IntoIden,
    IntoTableRef, TableCreateStatement,
};

impl Schema {
    /// Creates Postgres enums from an ActiveEnum. See [TypeCreateStatement] for more details
    pub fn create_enum_from_active_enum<A>(&self) -> TypeCreateStatement
    where
        A: ActiveEnum,
    {
        create_enum_from_active_enum::<A>(self.backend)
    }

    /// Creates Postgres enums from an Entity. See [TypeCreateStatement] for more details
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
        create_table_from_entity(entity, self.backend, self.schema_name.as_ref())
    }

    /// Creates the indexes from an Entity, returning an empty Vec if there are none
    /// to create. See [IndexCreateStatement] for more details
    pub fn create_index_from_entity<E>(&self, entity: E) -> Vec<IndexCreateStatement>
    where
        E: EntityTrait,
    {
        create_index_from_entity(entity, self.backend)
    }
}

pub(crate) fn create_enum_from_active_enum<A>(backend: DbBackend) -> TypeCreateStatement
where
    A: ActiveEnum,
{
    if matches!(backend, DbBackend::MySql | DbBackend::Sqlite) {
        panic!("TypeCreateStatement is not supported in MySQL & SQLite");
    }
    let col_def = A::db_type();
    let col_type = col_def.get_column_type();
    create_enum_from_column_type(col_type)
}

pub(crate) fn create_enum_from_column_type(col_type: &ColumnType) -> TypeCreateStatement {
    let (name, values) = match col_type {
        ColumnType::Enum(s, v) => (s.as_str(), v),
        _ => panic!("Should be ColumnType::Enum"),
    };
    Type::create()
        .as_enum(Alias::new(name))
        .values(values.iter().map(|val| Alias::new(val.as_str())))
        .to_owned()
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
        if !matches!(col_type, ColumnType::Enum(_, _)) {
            continue;
        }
        let stmt = create_enum_from_column_type(&col_type);
        vec.push(stmt);
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
    let mut vec = Vec::new();
    for column in E::Column::iter() {
        let column_def = column.def();
        if !column_def.indexed {
            continue;
        }
        let stmt = Index::create()
            .name(&format!(
                "idx-{}-{}",
                entity.to_string(),
                column.to_string()
            ))
            .table(entity)
            .col(column)
            .to_owned();
        vec.push(stmt)
    }
    vec
}

pub(crate) fn create_table_from_entity<E>(
    entity: E,
    backend: DbBackend,
    schema_name: Option<&String>,
) -> TableCreateStatement
where
    E: EntityTrait,
{
    let mut stmt = TableCreateStatement::new();

    for column in E::Column::iter() {
        let orm_column_def = column.def();
        let types = match orm_column_def.col_type {
            ColumnType::Enum(s, variants) => match backend {
                DbBackend::MySql => {
                    ColumnType::Custom(format!("ENUM('{}')", variants.join("', '")))
                }
                DbBackend::Postgres => ColumnType::Custom(s),
                DbBackend::Sqlite => ColumnType::Text,
            }
            .into(),
            _ => orm_column_def.col_type.into(),
        };
        let mut column_def = ColumnDef::new_with_type(column, types);
        if !orm_column_def.null {
            column_def.not_null();
        }
        if orm_column_def.unique {
            column_def.unique_key();
        }
        if let Some(value) = orm_column_def.default_value {
            column_def.default(value);
        }
        for primary_key in E::PrimaryKey::iter() {
            if column.to_string() == primary_key.into_column().to_string() {
                if E::PrimaryKey::auto_increment() {
                    column_def.auto_increment();
                }
                if E::PrimaryKey::iter().count() == 1 {
                    column_def.primary_key();
                }
            }
        }
        stmt.col(&mut column_def);
    }

    if E::PrimaryKey::iter().count() > 1 {
        let mut idx_pk = Index::create();
        for primary_key in E::PrimaryKey::iter() {
            idx_pk.col(primary_key);
        }
        stmt.primary_key(idx_pk.name(&format!("pk-{}", entity.to_string())).primary());
    }

    for relation in E::Relation::iter() {
        let relation = relation.def();
        if relation.is_owner {
            continue;
        }
        let mut foreign_key_stmt = ForeignKeyCreateStatement::new();
        let from_tbl = unpack_table_ref(&relation.from_tbl);
        let to_tbl = unpack_table_ref(&relation.to_tbl);
        let from_cols: Vec<String> = match relation.from_col {
            Identity::Unary(o1) => vec![o1],
            Identity::Binary(o1, o2) => vec![o1, o2],
            Identity::Ternary(o1, o2, o3) => vec![o1, o2, o3],
        }
        .into_iter()
        .map(|col| {
            let col_name = col.to_string();
            foreign_key_stmt.from_col(col);
            col_name
        })
        .collect();
        match relation.to_col {
            Identity::Unary(o1) => {
                foreign_key_stmt.to_col(o1);
            }
            Identity::Binary(o1, o2) => {
                foreign_key_stmt.to_col(o1);
                foreign_key_stmt.to_col(o2);
            }
            Identity::Ternary(o1, o2, o3) => {
                foreign_key_stmt.to_col(o1);
                foreign_key_stmt.to_col(o2);
                foreign_key_stmt.to_col(o3);
            }
        }
        if let Some(action) = relation.on_delete {
            foreign_key_stmt.on_delete(action);
        }
        if let Some(action) = relation.on_update {
            foreign_key_stmt.on_update(action);
        }
        let name = if let Some(name) = relation.fk_name {
            name
        } else {
            format!("fk-{}-{}", from_tbl.to_string(), from_cols.join("-"))
        };
        stmt.foreign_key(
            foreign_key_stmt
                .name(&name)
                .from_tbl(from_tbl)
                .to_tbl(to_tbl),
        );
    }

    // If table schema name is present, inject into `TableCreateStatement`
    if let Some(table_schema_name) = schema_name {
        stmt.table((Alias::new(&table_schema_name), entity.into_iden()).into_table_ref())
            .take()
    } else {
        stmt.table(entity.table_ref()).take()
    }
}

#[cfg(test)]
mod tests {
    use crate::{sea_query::*, tests_cfg::*, DbBackend, EntityName, Schema};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_create_table_from_entity_table_ref() {
        for builder in [DbBackend::MySql, DbBackend::Postgres, DbBackend::Sqlite] {
            let schema = Schema::new(builder, None);
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
                    .not_null(),
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
            let schema = Schema::new(builder, None);

            assert_eq!(
                builder.build(&schema.create_table_from_entity(indexes::Entity)),
                builder.build(
                    &get_indexes_stmt()
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
                .name("idx-indexes-index2_attr")
                .table(indexes::Entity)
                .col(indexes::Column::Index2Attr)
                .to_owned();
            assert_eq!(builder.build(&stmts[1]), builder.build(&idx));
        }
    }

    fn get_indexes_stmt() -> TableCreateStatement {
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
            .to_owned()
    }
}
