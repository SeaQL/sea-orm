use crate::{Column, Entity, EntityWriter, PrimaryKey, Relation};
use sea_orm::RelationType;
use sea_query::TableStatement;
use sea_schema::mysql::def::Schema;
use std::{collections::HashMap, mem::swap};

#[derive(Clone, Debug)]
pub struct EntityTransformer {
    pub(crate) schema: Schema,
}

impl EntityTransformer {
    pub fn transform(self) -> EntityWriter {
        let mut inverse_relations: HashMap<String, Vec<Relation>> = HashMap::new();
        let mut entities = Vec::new();

        for table_ref in self.schema.tables.iter() {
            let table_stmt = table_ref.write();
            // TODO: why return TableStatement?
            let table_create = match table_stmt {
                TableStatement::Create(stmt) => stmt,
                _ => panic!("TableStatement should be create"),
            };
            // println!("{:#?}", table_create);
            let table_name = match table_create.get_table_name() {
                Some(s) => s,
                None => panic!("Table name should not be empty"),
            };
            let columns = table_create.get_columns()
                .iter()
                .map(|col| {
                    let name = col.get_column_name();
                    let rs_type = "some_rust_type".to_string();
                    let col_type = match col.get_column_type() {
                        Some(ty) => ty.clone(),
                        None => panic!("ColumnType should not be empty"),
                    };
                    Column {
                        name,
                        rs_type,
                        col_type,
                    }
                })
                .collect();
            let relations = table_create.get_foreign_key_create_stmts()
                .iter()
                .map(|fk_stmt| fk_stmt.get_foreign_key())
                .map(|fk| {
                    let ref_table = match fk.get_ref_table() {
                        Some(s) => s,
                        None => panic!("RefTable should not be empty"),
                    };
                    let columns = fk.get_columns();
                    let ref_columns = fk.get_ref_columns();
                    let rel_type = RelationType::HasOne;
                    Relation {
                        ref_table,
                        columns,
                        ref_columns,
                        rel_type,
                    }
                });
            let primary_keys = table_create.get_indexes()
                .iter()
                .filter(|index| index.is_primary_key())
                .map(|index| index.get_index_spec()
                    .get_column_names()
                    .into_iter()
                    .map(|name| PrimaryKey { name })
                    .collect::<Vec<_>>()
                )
                .flatten()
                .collect();
            entities.push(Entity {
                table_name: table_name.clone(),
                columns,
                relations: relations.clone().collect(),
                primary_keys,
            });
            for mut rel in relations.into_iter() {
                let ref_table = rel.ref_table;
                swap(&mut rel.columns, &mut rel.ref_columns);
                rel.rel_type = RelationType::HasMany;
                rel.ref_table = table_name.clone();
                if let Some(vec) = inverse_relations.get_mut(&ref_table) {
                    vec.push(rel);
                } else {
                    inverse_relations.insert(ref_table, vec![rel]);
                }
            }
        }
        for (tbl_name, relations) in inverse_relations.iter() {
            for ent in entities.iter_mut() {
                if ent.table_name.eq(tbl_name) {
                    ent.relations.append(relations.clone().as_mut());
                }
            }
        }
        // println!();
        // println!("entities:");
        // println!("{:#?}", entities);
        // println!();
        // println!("inverse_relations:");
        // println!("{:#?}", inverse_relations);
        EntityWriter {
            entities,
        }
    }
}