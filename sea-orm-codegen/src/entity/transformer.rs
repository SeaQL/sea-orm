use crate::{
    ActiveEnum, Column, ConjunctRelation, Entity, EntityWriter, Error, PrimaryKey, Relation,
    RelationType,
};
use sea_query::TableStatement;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct EntityTransformer;

impl EntityTransformer {
    pub fn transform(table_stmts: Vec<TableStatement>) -> Result<EntityWriter, Error> {
        let mut enums: HashMap<String, ActiveEnum> = HashMap::new();
        let mut inverse_relations: HashMap<String, Vec<Relation>> = HashMap::new();
        let mut conjunct_relations: HashMap<String, Vec<ConjunctRelation>> = HashMap::new();
        let mut entities = HashMap::new();
        for table_stmt in table_stmts.into_iter() {
            let table_create = match table_stmt {
                TableStatement::Create(stmt) => stmt,
                _ => {
                    return Err(Error::TransformError(
                        "TableStatement should be create".into(),
                    ))
                }
            };
            let table_name = match table_create.get_table_name() {
                Some(table_ref) => match table_ref {
                    sea_query::TableRef::Table(t)
                    | sea_query::TableRef::SchemaTable(_, t)
                    | sea_query::TableRef::DatabaseTable(_, t)
                    | sea_query::TableRef::TableAlias(t, _)
                    | sea_query::TableRef::SchemaTableAlias(_, t, _)
                    | sea_query::TableRef::DatabaseTableAlias(_, t, _) => t.to_string(),
                    _ => unimplemented!(),
                },
                None => {
                    return Err(Error::TransformError(
                        "Table name should not be empty".into(),
                    ))
                }
            };
            let columns: Vec<Column> = table_create
                .get_columns()
                .iter()
                .map(|col_def| col_def.into())
                .map(|mut col: Column| {
                    col.unique = table_create
                        .get_indexes()
                        .iter()
                        .filter(|index| index.is_unique_key())
                        .map(|index| index.get_index_spec().get_column_names())
                        .filter(|col_names| col_names.len() == 1 && col_names[0] == col.name)
                        .count()
                        > 0;
                    col
                })
                .map(|col| {
                    if let sea_query::ColumnType::Enum(enum_name, values) = &col.col_type {
                        enums.insert(
                            enum_name.clone(),
                            ActiveEnum {
                                enum_name: enum_name.clone(),
                                values: values.clone(),
                            },
                        );
                    }
                    col
                })
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
                conjunct_relations: vec![],
                primary_keys,
            };
            entities.insert(table_name.clone(), entity.clone());
            for (i, mut rel) in relations.into_iter().enumerate() {
                let is_conjunct_relation = entity.primary_keys.len() == entity.columns.len()
                    && rel.columns.len() == 2
                    && rel.ref_columns.len() == 2
                    && entity.primary_keys.len() == 2;
                match is_conjunct_relation {
                    true => {
                        let another_rel = entity.relations.get((i == 0) as usize).unwrap();
                        let conjunct_relation = ConjunctRelation {
                            via: table_name.clone(),
                            to: another_rel.ref_table.clone(),
                        };
                        if let Some(vec) = conjunct_relations.get_mut(&rel.ref_table) {
                            vec.push(conjunct_relation);
                        } else {
                            conjunct_relations.insert(rel.ref_table, vec![conjunct_relation]);
                        }
                    }
                    false => {
                        let ref_table = rel.ref_table;
                        let mut unique = true;
                        for column in rel.columns.iter() {
                            if !entity
                                .columns
                                .iter()
                                .filter(|col| col.unique)
                                .any(|col| col.name.as_str() == column)
                            {
                                unique = false;
                                break;
                            }
                        }
                        let rel_type = if unique {
                            RelationType::HasOne
                        } else {
                            RelationType::HasMany
                        };
                        rel.rel_type = rel_type;
                        rel.ref_table = table_name.clone();
                        rel.columns = Vec::new();
                        rel.ref_columns = Vec::new();
                        if let Some(vec) = inverse_relations.get_mut(&ref_table) {
                            vec.push(rel);
                        } else {
                            inverse_relations.insert(ref_table, vec![rel]);
                        }
                    }
                }
            }
        }
        for (tbl_name, mut relations) in inverse_relations.into_iter() {
            if let Some(entity) = entities.get_mut(&tbl_name) {
                entity.relations.append(&mut relations);
            }
        }
        for (tbl_name, mut conjunct_relations) in conjunct_relations.into_iter() {
            if let Some(entity) = entities.get_mut(&tbl_name) {
                entity.conjunct_relations.append(&mut conjunct_relations);
            }
        }
        Ok(EntityWriter {
            entities: entities.into_iter().map(|(_, v)| v).collect(),
            enums,
        })
    }
}
