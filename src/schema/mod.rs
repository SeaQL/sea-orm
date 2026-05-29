use crate::DbBackend;

mod builder;
#[cfg(feature = "schema-sync")]
pub(crate) mod discover;
mod entity;
#[cfg(feature = "serde_json")]
mod json;
mod topology;

#[cfg(feature = "schema-sync")]
pub use discover::resolver;

pub use builder::*;
use topology::*;

/// This is a helper struct to convert [`EntityTrait`](crate::EntityTrait)
/// into different [`sea_query`](crate::sea_query) statements.
#[derive(Debug)]
pub struct Schema {
    //TODO: this struct is a wast
    backend: DbBackend,
}

impl Schema {
    /// Create a helper for a specific database backend
    pub fn new(backend: DbBackend) -> Self {
        Self { backend }
    }

    /// Creates a schema builder that can apply schema changes to database
    pub fn builder(self) -> SchemaBuilder {
        SchemaBuilder::new(self)
    }
}

// Sorts tables based on their foreign key dependencies
// pub(crate) fn sorted_tables(entities: &[builder::EntitySchemaInfo]) -> Vec<sea_query::TableName> {
//     let mut sorter = TopologicalSort::<sea_query::TableName>::new();

//     for entity in entities.iter() {
//         let table_name = builder::get_table_name(entity.table().get_table_name());
//         sorter.insert(table_name);
//     }
//     for entity in entities.iter() {
//         let self_table = builder::get_table_name(entity.table().get_table_name());
//         for fk in entity.table().get_foreign_key_create_stmts().iter() {
//             let fk = fk.get_foreign_key();
//             let ref_table = builder::get_table_name(fk.get_ref_table());
//             if self_table != ref_table {
//                 // self cycle is okay
//                 sorter.add_dependency(ref_table, self_table.clone());
//             }
//         }
//     }
//     let mut sorted = Vec::new();
//     while let Some(i) = sorter.pop() {
//         sorted.push(i);
//     }
//     if sorted.len() != entities.len() {
//         // push leftover tables
//         for entity in entities.iter() {
//             let table_name = builder::get_table_name(entity.table().get_table_name());
//             if !sorted.contains(&table_name) {
//                 sorted.push(table_name);
//             }
//         }
//     }

//     sorted
// }
