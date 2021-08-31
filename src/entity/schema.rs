use crate::{ColumnDef as Sea_Orm_Column_Def, EntityTrait, PrimaryKeyTrait, RelationDef};
use crate::entity::column::ColumnTrait;
use crate::entity::relation::RelationTrait;
use crate::entity::primary_key::PrimaryKeyToColumn;
use sea_query::{Alias, ColumnDef as Sea_Query_Column_Def, ColumnType as Sea_Query_Column_Type, ForeignKeyCreateStatement, IndexCreateStatement, SeaRc, TableCreateStatement, TableRef};
pub use sea_strum::IntoEnumIterator;
pub trait CreateStatementOf
{
    fn create_table_statement_of<E>(entity: E) -> TableCreateStatement where E: EntityTrait {
        let mut stmt = TableCreateStatement::new();
        stmt.table(entity);
        for relation in E::Relation::iter() {
            let mut foreign_key_stmt = ForeignKeyCreateStatement::new();
            let relation_trait: RelationDef = relation.def();
            // foreign_key_stmt.name("Temp");
            match relation_trait.from_tbl {
                TableRef::Table(tbl) => { foreign_key_stmt.from_tbl(tbl); },
                _ => todo!(),
            }
            match relation_trait.to_tbl {
                TableRef::Table(tbl) => { foreign_key_stmt.to_tbl(tbl); },
                _ => todo!(),
            }
            match relation_trait.from_col {
                crate::Identity::Unary(o1) => {
                    foreign_key_stmt.from_col(o1);
                },
                crate::Identity::Binary(o1, o2) => {
                    foreign_key_stmt.from_col(o1);
                    foreign_key_stmt.from_col(o2);
                },
                crate::Identity::Ternary(o1, o2, o3) => {
                    foreign_key_stmt.from_col(o1);
                    foreign_key_stmt.from_col(o2);
                    foreign_key_stmt.from_col(o3);
                },
            }
            match relation_trait.to_col {
                crate::Identity::Unary(o1) => {
                    foreign_key_stmt.to_col(o1);
                },
                crate::Identity::Binary(o1, o2) => {
                    foreign_key_stmt.to_col(o1);
                    foreign_key_stmt.to_col(o2);
                },
                crate::Identity::Ternary(o1, o2, o3) => {
                    foreign_key_stmt.to_col(o1);
                    foreign_key_stmt.to_col(o2);
                    foreign_key_stmt.to_col(o3);
                },
            }
            stmt.foreign_key(&mut foreign_key_stmt);
        }
        for col in E::Column::iter() {
            let sea_orm_column_def: Sea_Orm_Column_Def = col.def().into();
            let mut index = IndexCreateStatement::new();
            let mut sea_query_column_def = Sea_Query_Column_Def::new(col);
            for key in E::PrimaryKey::iter() { // enum: Id, Name ...
                if sea_query_column_def.get_column_name() == Sea_Query_Column_Def::new(key.into_column()).get_column_name() {
                    sea_query_column_def.primary_key();
                    if E::PrimaryKey::auto_increment() {
                        sea_query_column_def.auto_increment();
                    }
                    index.primary();
                }
            }
            if !sea_orm_column_def.null {
                sea_query_column_def.not_null();
            }
            if sea_orm_column_def.unique {
                sea_query_column_def.unique_key();
                index.unique();
            }
            if sea_orm_column_def.indexed {
                index.table(entity);
                index.col(col);
                stmt.index(&mut index);
            }
            match Sea_Query_Column_Type::from(sea_orm_column_def.col_type) {
                Sea_Query_Column_Type::Char(length) => match length {
                    Some(length) => { sea_query_column_def.char_len(length); },
                    None => { sea_query_column_def.char(); },
                },
                Sea_Query_Column_Type::String(length) => match length {
                    Some(length) => { sea_query_column_def.string_len(length); },
                    None => { sea_query_column_def.string(); },
                },
                Sea_Query_Column_Type::Text => { sea_query_column_def.text(); },
                Sea_Query_Column_Type::TinyInteger(length) => match length {
                    Some(length) => { sea_query_column_def.tiny_integer_len(length); },
                    None => { sea_query_column_def.tiny_integer(); },
                },
                // Sea_Query_Column_Type::TinyInteger => { sea_query_column_def.tiny_integer(); },
                Sea_Query_Column_Type::SmallInteger(length) => match length {
                    Some(length) => { sea_query_column_def.small_integer_len(length); },
                    None => { sea_query_column_def.small_integer(); },
                },
                Sea_Query_Column_Type::Integer(length) => match length {
                    Some(length) => { sea_query_column_def.integer_len(length); },
                    None => { sea_query_column_def.integer(); },
                },
                Sea_Query_Column_Type::BigInteger(length) => match length {
                    Some(length) => { sea_query_column_def.big_integer_len(length); },
                    None => { sea_query_column_def.big_integer(); },
                },
                Sea_Query_Column_Type::Float(precision) => match precision {
                    Some(precision) => { sea_query_column_def.float_len(precision); },
                    None => { sea_query_column_def.float(); },
                },
                Sea_Query_Column_Type::Double(precision) => match precision {
                    Some(precision) => { sea_query_column_def.double_len(precision); },
                    None => { sea_query_column_def.double(); },
                },
                Sea_Query_Column_Type::Decimal(_) =>  { sea_query_column_def.decimal(); },
                Sea_Query_Column_Type::DateTime(precision) => match precision {
                    Some(precision) => { sea_query_column_def.date_time_len(precision); },
                    None => { sea_query_column_def.date_time(); },
                },
                Sea_Query_Column_Type::Timestamp(precision) => match precision {
                    Some(precision) => { sea_query_column_def.timestamp_len(precision); },
                    None => { sea_query_column_def.timestamp(); },
                },
                Sea_Query_Column_Type::Time(precision) => match precision {
                    Some(precision) => { sea_query_column_def.time_len(precision); },
                    None => { sea_query_column_def.time(); },
                },
                Sea_Query_Column_Type::Date =>  { sea_query_column_def.date(); },
                Sea_Query_Column_Type::Binary(length) => match length {
                    Some(length) => { sea_query_column_def.binary_len(length); },
                    None => { sea_query_column_def.binary(); },
                },
                Sea_Query_Column_Type::Boolean =>  { sea_query_column_def.boolean(); },
                Sea_Query_Column_Type::Money(_) =>  { sea_query_column_def.money(); },
                Sea_Query_Column_Type::Json =>  { sea_query_column_def.json(); },
                Sea_Query_Column_Type::JsonBinary =>  { sea_query_column_def.json_binary(); },
                Sea_Query_Column_Type::Custom(iden) => { sea_query_column_def.custom(Alias::new(&iden.to_string())); },
                Sea_Query_Column_Type::Uuid =>  { sea_query_column_def.uuid(); },
                Sea_Query_Column_Type::TimestampWithTimeZone(length) => match length {
                    Some(length) => { sea_query_column_def.timestamp_with_time_zone_len(length); },
                    None => { sea_query_column_def.timestamp_with_time_zone(); },
                },
            }
            stmt.col(&mut sea_query_column_def);
        }
        stmt.if_not_exists();

        stmt
    }
}

impl<EntityTrait> CreateStatementOf for EntityTrait {}

#[cfg(test)]
mod tests {
    use crate::{CreateStatementOf, tests_cfg};

    #[test]
    fn test_create_statement_tests_cfg_cake() {
        let create_statement = tests_cfg::cake::Entity::create_table_statement_of(tests_cfg::cake::Entity);
        let table = format!("{:?}", create_statement.get_table_name());
        let columns = format!("{:?}", create_statement.get_columns());
        let relations = format!("{:?}", create_statement.get_foreign_key_create_stmts());
        let indexs = format!("{:?}", create_statement.get_indexes());
        let result = format!("{:?}", create_statement);
        assert_eq!("TableCreateStatement { table: Some(cake), columns: [ColumnDef { table: Some(cake), name: id, types: Some(Integer(None)), spec: [PrimaryKey, AutoIncrement, NotNull] }, ColumnDef { table: Some(cake), name: name, types: Some(String(None)), spec: [NotNull] }], options: [], partitions: [], indexes: [], foreign_keys: [ForeignKeyCreateStatement { foreign_key: TableForeignKey { name: None, table: Some(cake), ref_table: Some(fruit), columns: [id], ref_columns: [cake_id], on_delete: None, on_update: None } }], if_not_exists: true }", result);
        assert_eq!(r#"Some("cake")"#, table);
        assert_eq!("[ForeignKeyCreateStatement { foreign_key: TableForeignKey { name: None, table: Some(cake), ref_table: Some(fruit), columns: [id], ref_columns: [cake_id], on_delete: None, on_update: None } }]", relations);
        assert_eq!(r#"[ColumnDef { table: Some(cake), name: id, types: Some(Integer(None)), spec: [PrimaryKey, AutoIncrement, NotNull] }, ColumnDef { table: Some(cake), name: name, types: Some(String(None)), spec: [NotNull] }]"#, columns);
        assert_eq!("[]", indexs);
    }
}
