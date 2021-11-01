use crate::{
    unpack_table_ref, ColumnTrait, EntityTrait, Identity, Iterable, PrimaryKeyToColumn,
    PrimaryKeyTrait, RelationTrait, Schema,
};
use sea_query::{ColumnDef, ForeignKeyCreateStatement, Iden, Index, TableCreateStatement};

impl Schema {
    /// Creates a table from an Entity. See [TableCreateStatement] for more details
    pub fn create_table_from_entity<E>(entity: E) -> TableCreateStatement
    where
        E: EntityTrait,
    {
        create_table_from_entity(entity)
    }
}

pub(crate) fn create_table_from_entity<E>(entity: E) -> TableCreateStatement
where
    E: EntityTrait,
{
    let mut stmt = TableCreateStatement::new();

    for column in E::Column::iter() {
        let orm_column_def = column.def();
        let types = orm_column_def.col_type.into();
        let mut column_def = ColumnDef::new_with_type(column, types);
        if !orm_column_def.null {
            column_def.not_null();
        }
        if orm_column_def.unique {
            column_def.unique_key();
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
        if orm_column_def.indexed {
            stmt.index(
                Index::create()
                    .name(&format!(
                        "idx-{}-{}",
                        entity.to_string(),
                        column.to_string()
                    ))
                    .table(entity)
                    .col(column),
            );
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
        match relation.from_col {
            Identity::Unary(o1) => {
                foreign_key_stmt.from_col(o1);
            }
            Identity::Binary(o1, o2) => {
                foreign_key_stmt.from_col(o1);
                foreign_key_stmt.from_col(o2);
            }
            Identity::Ternary(o1, o2, o3) => {
                foreign_key_stmt.from_col(o1);
                foreign_key_stmt.from_col(o2);
                foreign_key_stmt.from_col(o3);
            }
        }
        match relation.to_col {
            Identity::Unary(o1) => {
                foreign_key_stmt.to_col(o1);
            }
            Identity::Binary(o1, o2) => {
                foreign_key_stmt.to_col(o1);
                foreign_key_stmt.to_col(o2);
            }
            crate::Identity::Ternary(o1, o2, o3) => {
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
        stmt.foreign_key(
            foreign_key_stmt
                .name(&format!(
                    "fk-{}-{}",
                    from_tbl.to_string(),
                    to_tbl.to_string()
                ))
                .from_tbl(from_tbl)
                .to_tbl(to_tbl),
        );
    }

    stmt.table(entity).take()
}

#[cfg(test)]
mod tests {
    use crate::{sea_query::*, tests_cfg::*, Schema};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_create_table_from_entity() {
        assert_eq!(
            Schema::create_table_from_entity(CakeFillingPrice).to_string(MysqlQueryBuilder),
            Table::create()
                .table(CakeFillingPrice)
                .col(
                    ColumnDef::new(cake_filling_price::Column::CakeId)
                        .integer()
                        .not_null()
                )
                .col(
                    ColumnDef::new(cake_filling_price::Column::FillingId)
                        .integer()
                        .not_null()
                )
                .col(
                    ColumnDef::new(cake_filling_price::Column::Price)
                        .decimal()
                        .not_null()
                )
                .primary_key(
                    Index::create()
                        .name("pk-cake_filling_price")
                        .col(cake_filling_price::Column::CakeId)
                        .col(cake_filling_price::Column::FillingId)
                        .primary()
                )
                .foreign_key(
                    ForeignKeyCreateStatement::new()
                        .name("fk-cake_filling_price-cake_filling")
                        .from_tbl(CakeFillingPrice)
                        .from_col(cake_filling_price::Column::CakeId)
                        .from_col(cake_filling_price::Column::FillingId)
                        .to_tbl(CakeFilling)
                        .to_col(cake_filling::Column::CakeId)
                        .to_col(cake_filling::Column::FillingId)
                )
                .to_string(MysqlQueryBuilder)
        );
    }
}
