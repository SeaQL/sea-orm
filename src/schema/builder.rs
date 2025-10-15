use super::{Schema, TopologicalSort};
use crate::{ConnectionTrait, DbBackend, DbErr, EntityTrait};
use sea_query::{
    IndexCreateStatement, TableCreateStatement, TableName, TableRef,
    extension::postgres::TypeCreateStatement,
};

/// A schema builder that can take a registry of Entities and synchronize it with database.
pub struct SchemaBuilder {
    helper: Schema,
    entities: Vec<EntitySchema>,
}

struct EntitySchema {
    table: TableCreateStatement,
    enums: Vec<TypeCreateStatement>,
    indexes: Vec<IndexCreateStatement>,
}

impl std::fmt::Debug for SchemaBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SchemaBuilder {{")?;
        write!(f, " entities: [")?;
        for (i, entity) in self.entities.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            entity.debug_print(f, &self.helper.backend)?;
        }
        write!(f, " ]")?;
        write!(f, " }}")
    }
}

impl SchemaBuilder {
    /// Creates a new schema builder
    pub fn new(schema: Schema) -> Self {
        Self {
            helper: schema,
            entities: Default::default(),
        }
    }

    /// Register an entity to this schema
    pub fn register<E: EntityTrait>(mut self, entity: E) -> Self {
        self.entities.push(EntitySchema {
            table: self.helper.create_table_from_entity(entity),
            enums: self.helper.create_enum_from_entity(entity),
            indexes: self.helper.create_index_from_entity(entity),
        });
        self
    }

    /// Synchronize the schema with database, will create missing tables, indexes and columns.
    /// This operation is addition only, will not drop anything.
    /// It will attempt to alter column types by casting to string, but will abort if failing.
    pub async fn sync<C: ConnectionTrait>(self, db: &C) -> Result<(), DbErr> {
        let mut sorter = TopologicalSort::<TableName>::new();

        for entity in self.entities.iter() {
            let table_name = get_table_name(entity.table.get_table_name());
            sorter.insert(table_name);
        }
        for entity in self.entities.iter() {
            let self_table = get_table_name(entity.table.get_table_name());
            for fk in entity.table.get_foreign_key_create_stmts().iter() {
                let fk = fk.get_foreign_key();
                let ref_table = get_table_name(fk.get_ref_table());
                if self_table != ref_table {
                    // self cycle is okay
                    sorter.add_dependency(self_table.clone(), ref_table);
                }
            }
        }
        let mut sorted = Vec::new();
        while let Some(i) = sorter.pop() {
            sorted.push(i);
        }
        if sorted.len() != self.entities.len() {
            // push leftover tables
            for entity in self.entities.iter() {
                let table_name = get_table_name(entity.table.get_table_name());
                if !sorted.contains(&table_name) {
                    sorted.push(table_name);
                }
            }
        }

        for table_name in sorted {
            if let Some(entity) = self
                .entities
                .iter()
                .find(|entity| table_name == get_table_name(entity.table.get_table_name()))
            {
                entity.apply(db).await?;
            }
        }

        Ok(())
    }
}

impl EntitySchema {
    async fn apply<C: ConnectionTrait>(&self, db: &C) -> Result<(), DbErr> {
        // TODO check if table exists
        db.execute(&self.table).await?;
        for stmt in self.indexes.iter() {
            // TODO check if index exists
            db.execute(stmt).await?;
        }
        for stmt in self.enums.iter() {
            // TODO check if index exists
            db.execute(stmt).await?;
        }
        Ok(())
    }

    fn debug_print(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        backend: &DbBackend,
    ) -> std::fmt::Result {
        write!(f, "EntitySchema {{")?;
        write!(f, " table: {:?}", backend.build(&self.table).to_string())?;
        write!(f, " enums: [")?;
        for (i, stmt) in self.enums.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:?}", backend.build(stmt).to_string())?;
        }
        write!(f, " ]")?;
        write!(f, " indexes: [")?;
        for (i, stmt) in self.indexes.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:?}", backend.build(stmt).to_string())?;
        }
        write!(f, " ]")?;
        write!(f, " }}")
    }
}

fn get_table_name(table_ref: Option<&TableRef>) -> TableName {
    match table_ref {
        Some(TableRef::Table(table_name, _)) => table_name.clone(),
        None => panic!("Expect TableCreateStatement is properly built"),
        _ => unreachable!("Unexpected {table_ref:?}"),
    }
}
