use crate::{
    ActiveEnum, Column, ConjunctRelation, Entity, EntityWriter, Error, PrimaryKey, Relation,
    RelationType,
};
use sea_query::{ColumnSpec, TableCreateStatement};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct EntityTransformer;

impl EntityTransformer {
    pub fn transform(table_create_stmts: Vec<TableCreateStatement>) -> Result<EntityWriter, Error> {
        let mut enums: HashMap<String, ActiveEnum> = HashMap::new();
        let mut inverse_relations: HashMap<String, Vec<Relation>> = HashMap::new();
        let mut conjunct_relations: HashMap<String, Vec<ConjunctRelation>> = HashMap::new();
        let mut entities = HashMap::new();
        for table_create in table_create_stmts.into_iter() {
            let table_name = match table_create.get_table_name() {
                Some(table_ref) => match table_ref {
                    sea_query::TableRef::Table(t)
                    | sea_query::TableRef::SchemaTable(_, t)
                    | sea_query::TableRef::DatabaseSchemaTable(_, _, t)
                    | sea_query::TableRef::TableAlias(t, _)
                    | sea_query::TableRef::SchemaTableAlias(_, t, _)
                    | sea_query::TableRef::DatabaseSchemaTableAlias(_, _, t, _) => t.to_string(),
                    _ => unimplemented!(),
                },
                None => {
                    return Err(Error::TransformError(
                        "Table name should not be empty".into(),
                    ))
                }
            };
            let mut primary_keys: Vec<PrimaryKey> = Vec::new();
            let columns: Vec<Column> = table_create
                .get_columns()
                .iter()
                .map(|col_def| {
                    let primary_key = col_def
                        .get_column_spec()
                        .iter()
                        .any(|spec| matches!(spec, ColumnSpec::PrimaryKey));
                    if primary_key {
                        primary_keys.push(PrimaryKey {
                            name: col_def.get_column_name(),
                        });
                    }
                    col_def.into()
                })
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
            let mut ref_table_counts: HashMap<String, usize> = HashMap::new();
            let relations: Vec<Relation> = table_create
                .get_foreign_key_create_stmts()
                .iter()
                .map(|fk_create_stmt| fk_create_stmt.get_foreign_key())
                .map(|tbl_fk| {
                    let ref_tbl = tbl_fk.get_ref_table().unwrap();
                    if let Some(count) = ref_table_counts.get_mut(&ref_tbl) {
                        if *count == 0 {
                            *count = 1;
                        }
                        *count += 1;
                    } else {
                        ref_table_counts.insert(ref_tbl, 0);
                    };
                    tbl_fk.into()
                })
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .map(|mut rel: Relation| {
                    rel.self_referencing = rel.ref_table == table_name;
                    if let Some(count) = ref_table_counts.get_mut(&rel.ref_table) {
                        rel.num_suffix = *count;
                        if *count > 0 {
                            *count -= 1;
                        }
                    }
                    rel
                })
                .rev()
                .collect();
            primary_keys.extend(
                table_create
                    .get_indexes()
                    .iter()
                    .filter(|index| index.is_primary_key())
                    .flat_map(|index| {
                        index
                            .get_index_spec()
                            .get_column_names()
                            .into_iter()
                            .map(|name| PrimaryKey { name })
                            .collect::<Vec<_>>()
                    }),
            );
            let entity = Entity {
                table_name: table_name.clone(),
                columns,
                relations: relations.clone(),
                conjunct_relations: vec![],
                primary_keys,
            };
            entities.insert(table_name.clone(), entity.clone());
            for (i, mut rel) in relations.into_iter().enumerate() {
                // This will produce a duplicated relation
                if rel.self_referencing {
                    continue;
                }
                // This will cause compile error on the many side,
                // got relation variant but without Related<T> implemented
                if rel.num_suffix > 0 {
                    continue;
                }
                let is_conjunct_relation = entity.primary_keys.len() == entity.columns.len()
                    && entity.relations.len() == 2
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
        for (tbl_name, relations) in inverse_relations.into_iter() {
            if let Some(entity) = entities.get_mut(&tbl_name) {
                for relation in relations.into_iter() {
                    let duplicate_relation = entity
                        .relations
                        .iter()
                        .any(|rel| rel.ref_table == relation.ref_table);
                    if !duplicate_relation {
                        entity.relations.push(relation);
                    }
                }
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
