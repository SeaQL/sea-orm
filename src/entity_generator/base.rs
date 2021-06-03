use crate::{ColumnSpec, EntitySpec, RelationSpec, RelationType};
use sea_query::ColumnType;
use sea_schema::mysql::{def::{ColumnKey, Schema}, discovery::SchemaDiscovery};
use sqlx::MySqlPool;
use std::{collections::HashMap, mem::swap};

#[derive(Clone, Debug)]
pub struct EntityGenerator {
    pub(crate) schema: Schema,
    pub(crate) entities: Vec<EntitySpec>,
    pub(crate) inverse_relations: HashMap<String, Vec<RelationSpec>>,
}

impl EntityGenerator {
    pub async fn new(uri: &str, schema: &str) -> Self {
        Self {
            schema: Self::discover(uri, schema).await,
            entities: Vec::new(),
            inverse_relations: HashMap::new(),
        }
    }

    pub async fn discover(uri: &str, schema: &str) -> Schema {
        let connection = MySqlPool::connect(uri).await.unwrap();
        let schema_discovery = SchemaDiscovery::new(connection, schema);
        schema_discovery.discover().await
    }

    pub fn parse(mut self) -> Self {
        for table_ref in self.schema.tables.iter() {
            let table_name = table_ref.info.name.clone();
            let columns = table_ref.columns
                .iter()
                .map(|col_info| {
                    ColumnSpec {
                        name: col_info.name.clone(),
                        rs_type: "some_rust_type".to_string(),
                        col_type: ColumnType::String(None),
                        is_primary_key: match &col_info.key {
                            ColumnKey::Primary => true,
                            _ => false,
                        },
                    }
                })
                .collect();
            let relations = table_ref.foreign_keys
                .iter()
                .map(|fk_info| {
                    RelationSpec {
                        ref_table: fk_info.referenced_table.clone(),
                        columns: fk_info.columns.clone(),
                        ref_columns: fk_info.referenced_columns.clone(),
                        rel_type: RelationType::HasOne,
                    }
                });
            self.entities.push(EntitySpec {
                table_name: table_name.clone(),
                columns,
                relations: relations.clone().collect(),
            });
            for mut rel in relations.into_iter() {
                let ref_table = rel.ref_table;
                swap(&mut rel.columns, &mut rel.ref_columns);
                rel.rel_type = RelationType::HasMany;
                rel.ref_table = table_name.clone();
                if let Some(vec) = self.inverse_relations.get_mut(&ref_table) {
                    vec.push(rel);
                } else {
                    self.inverse_relations.insert(ref_table, vec![rel]);
                }
            }
        }
        println!();
        println!("entities:");
        println!("{:#?}", self.entities);
        println!();
        println!("inverse_relations:");
        println!("{:#?}", self.inverse_relations);
        self
    }
}
