use crate::{Entity, EntityWriter, Error, PrimaryKey, Relation};
use sea_orm::RelationType;
use sea_query::TableStatement;
use sea_schema::mysql::def::Schema;
use std::{collections::HashMap, mem::swap};

#[derive(Clone, Debug)]
pub struct EntityTransformer {
    pub(crate) schema: Schema,
}

impl EntityTransformer {
    pub fn transform(self) -> Result<EntityWriter, Error> {
        let mut inverse_relations: HashMap<String, Vec<Relation>> = HashMap::new();
        let mut entities = Vec::new();
        for table_ref in self.schema.tables.iter() {
            let table_stmt = table_ref.write();
            let table_create = match table_stmt {
                TableStatement::Create(stmt) => stmt,
                _ => {
                    return Err(Error::TransformError(
                        "TableStatement should be create".into(),
                    ))
                }
            };
            let table_name = match table_create.get_table_name() {
                Some(s) => s,
                None => {
                    return Err(Error::TransformError(
                        "Table name should not be empty".into(),
                    ))
                }
            };
            let columns = table_create
                .get_columns()
                .iter()
                .map(|col_def| col_def.into())
                .collect();
            let relations = table_create
                .get_foreign_key_create_stmts()
                .iter()
                .map(|fk_create_stmt| fk_create_stmt.get_foreign_key())
                .map(|tbl_fk| tbl_fk.into());
            let primary_keys = table_create
                .get_indexes()
                .iter()
                .filter(|index| index.is_primary_key())
                .map(|index| {
                    index
                        .get_index_spec()
                        .get_column_names()
                        .into_iter()
                        .map(|name| PrimaryKey { name })
                        .collect::<Vec<_>>()
                })
                .flatten()
                .collect();
            let entity = Entity {
                table_name: table_name.clone(),
                columns,
                relations: relations.clone().collect(),
                primary_keys,
            };
            entities.push(entity);
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
        Ok(EntityWriter { entities })
    }
}
