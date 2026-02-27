use super::{Schema, TopologicalSort};
use crate::{ConnectionTrait, DbBackend, DbErr, EntityTrait, Statement};
use sea_query::{
    ForeignKeyCreateStatement, Index, IndexCreateStatement, IntoIden, TableAlterStatement,
    TableCreateStatement, TableName, TableRef, extension::postgres::TypeCreateStatement,
};

/// A schema builder that can take a registry of Entities and synchronize it with database.
pub struct SchemaBuilder {
    helper: Schema,
    entities: Vec<EntitySchemaInfo>,
}

/// Schema info for Entity. Can be used to re-create schema in database.
pub struct EntitySchemaInfo {
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

impl std::fmt::Debug for EntitySchemaInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.debug_print(f, &DbBackend::Sqlite)
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
        let entity = EntitySchemaInfo::new(entity, &self.helper);
        if !self
            .entities
            .iter()
            .any(|e| e.table.get_table_name() == entity.table.get_table_name())
        {
            self.entities.push(entity);
        }
        self
    }

    #[cfg(feature = "entity-registry")]
    pub(crate) fn helper(&self) -> &Schema {
        &self.helper
    }

    #[cfg(feature = "entity-registry")]
    pub(crate) fn register_entity(&mut self, entity: EntitySchemaInfo) {
        self.entities.push(entity);
    }

    /// Synchronize the schema with database, will create missing tables, columns, unique keys, and foreign keys.
    /// This operation is addition only, will not drop any table / columns.
    #[cfg(feature = "schema-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "schema-sync")))]
    pub fn sync<C>(self, db: &C) -> Result<(), DbErr>
    where
        C: ConnectionTrait + sea_schema::Connection,
    {
        let _existing = match db.get_database_backend() {
            #[cfg(feature = "sqlx-mysql")]
            DbBackend::MySql => {
                use sea_schema::{mysql::discovery::SchemaDiscovery, probe::SchemaProbe};

                let current_schema: String = db
                    .query_one(
                        sea_query::SelectStatement::new()
                            .expr(sea_schema::mysql::MySql::get_current_schema()),
                    )?
                    .ok_or_else(|| DbErr::RecordNotFound("Can't get current schema".into()))?
                    .try_get_by_index(0)?;
                let schema_discovery = SchemaDiscovery::new_no_exec(&current_schema);

                let schema = schema_discovery
                    .discover_with(db)
                    .map_err(|err| DbErr::Query(crate::RuntimeErr::SqlxError(err.into())))?;

                DiscoveredSchema {
                    tables: schema.tables.iter().map(|table| table.write()).collect(),
                    enums: vec![],
                }
            }
            #[cfg(feature = "sqlx-postgres")]
            DbBackend::Postgres => {
                use sea_schema::{postgres::discovery::SchemaDiscovery, probe::SchemaProbe};

                let current_schema: String = db
                    .query_one(
                        sea_query::SelectStatement::new()
                            .expr(sea_schema::postgres::Postgres::get_current_schema()),
                    )?
                    .ok_or_else(|| DbErr::RecordNotFound("Can't get current schema".into()))?
                    .try_get_by_index(0)?;
                let schema_discovery = SchemaDiscovery::new_no_exec(&current_schema);

                let schema = schema_discovery
                    .discover_with(db)
                    .map_err(|err| DbErr::Query(crate::RuntimeErr::SqlxError(err.into())))?;

                DiscoveredSchema {
                    tables: schema.tables.iter().map(|table| table.write()).collect(),
                    enums: schema.enums.iter().map(|def| def.write()).collect(),
                }
            }
            #[cfg(feature = "sqlx-sqlite")]
            DbBackend::Sqlite => {
                use sea_schema::sqlite::{SqliteDiscoveryError, discovery::SchemaDiscovery};
                let schema = SchemaDiscovery::discover_with(db)
                    .map_err(|err| {
                        DbErr::Query(match err {
                            SqliteDiscoveryError::SqlxError(err) => {
                                crate::RuntimeErr::SqlxError(err.into())
                            }
                            _ => crate::RuntimeErr::Internal(format!("{err:?}")),
                        })
                    })?
                    .merge_indexes_into_table();
                DiscoveredSchema {
                    tables: schema.tables.iter().map(|table| table.write()).collect(),
                    enums: vec![],
                }
            }
            #[cfg(feature = "rusqlite")]
            DbBackend::Sqlite => {
                use sea_schema::sqlite::{SqliteDiscoveryError, discovery::SchemaDiscovery};
                let schema = SchemaDiscovery::discover_with(db)
                    .map_err(|err| {
                        DbErr::Query(match err {
                            SqliteDiscoveryError::RusqliteError(err) => {
                                crate::RuntimeErr::Rusqlite(err.into())
                            }
                            _ => crate::RuntimeErr::Internal(format!("{err:?}")),
                        })
                    })?
                    .merge_indexes_into_table();
                DiscoveredSchema {
                    tables: schema.tables.iter().map(|table| table.write()).collect(),
                    enums: vec![],
                }
            }
            #[allow(unreachable_patterns)]
            other => {
                return Err(DbErr::BackendNotSupported {
                    db: other.as_str(),
                    ctx: "SchemaBuilder::sync",
                });
            }
        };

        #[allow(unreachable_code)]
        let mut created_enums: Vec<Statement> = Default::default();

        #[allow(unreachable_code)]
        for table_name in self.sorted_tables() {
            if let Some(entity) = self
                .entities
                .iter()
                .find(|entity| table_name == get_table_name(entity.table.get_table_name()))
            {
                entity.sync(db, &_existing, &mut created_enums)?;
            }
        }

        Ok(())
    }

    /// Apply this schema to a database, will create all registered tables, columns, unique keys, and foreign keys.
    /// Will fail if any table already exists. Use [`sync`] if you want an incremental version that can perform schema diff.
    pub fn apply<C: ConnectionTrait>(self, db: &C) -> Result<(), DbErr> {
        let mut created_enums: Vec<Statement> = Default::default();

        for table_name in self.sorted_tables() {
            if let Some(entity) = self
                .entities
                .iter()
                .find(|entity| table_name == get_table_name(entity.table.get_table_name()))
            {
                entity.apply(db, &mut created_enums)?;
            }
        }

        Ok(())
    }

    fn sorted_tables(&self) -> Vec<TableName> {
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
                    sorter.add_dependency(ref_table, self_table.clone());
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

        sorted
    }
}

struct DiscoveredSchema {
    tables: Vec<TableCreateStatement>,
    enums: Vec<TypeCreateStatement>,
}

impl EntitySchemaInfo {
    /// Creates a EntitySchemaInfo object given a generic Entity.
    pub fn new<E: EntityTrait>(entity: E, helper: &Schema) -> Self {
        Self {
            table: helper.create_table_from_entity(entity),
            enums: helper.create_enum_from_entity(entity),
            indexes: helper.create_index_from_entity(entity),
        }
    }

    fn apply<C: ConnectionTrait>(
        &self,
        db: &C,
        created_enums: &mut Vec<Statement>,
    ) -> Result<(), DbErr> {
        for stmt in self.enums.iter() {
            let new_stmt = db.get_database_backend().build(stmt);
            if !created_enums.iter().any(|s| s == &new_stmt) {
                db.execute(stmt)?;
                created_enums.push(new_stmt);
            }
        }
        db.execute(&self.table)?;
        for stmt in self.indexes.iter() {
            db.execute(stmt)?;
        }
        Ok(())
    }

    // better to always compile this function
    #[allow(dead_code)]
    fn sync<C: ConnectionTrait>(
        &self,
        db: &C,
        existing: &DiscoveredSchema,
        created_enums: &mut Vec<Statement>,
    ) -> Result<(), DbErr> {
        let db_backend = db.get_database_backend();

        // create enum before creating table
        for stmt in self.enums.iter() {
            let mut has_enum = false;
            let new_stmt = db_backend.build(stmt);
            for existing_enum in &existing.enums {
                if db_backend.build(existing_enum) == new_stmt {
                    has_enum = true;
                    // TODO add enum variants
                    break;
                }
            }
            if !has_enum && !created_enums.iter().any(|s| s == &new_stmt) {
                db.execute(stmt)?;
                created_enums.push(new_stmt);
            }
        }
        let table_name = get_table_name(self.table.get_table_name());
        let mut existing_table = None;
        for tbl in &existing.tables {
            if get_table_name(tbl.get_table_name()) == table_name {
                existing_table = Some(tbl);
                break;
            }
        }
        if let Some(existing_table) = existing_table {
            for column_def in self.table.get_columns() {
                let mut column_exists = false;
                for existing_column in existing_table.get_columns() {
                    if column_def.get_column_name() == existing_column.get_column_name() {
                        column_exists = true;
                        break;
                    }
                }
                if !column_exists {
                    let mut renamed_from = "";
                    if let Some(comment) = &column_def.get_column_spec().comment {
                        if let Some((_, suffix)) = comment.rsplit_once("renamed_from \"") {
                            if let Some((prefix, _)) = suffix.split_once('"') {
                                renamed_from = prefix;
                            }
                        }
                    }
                    if renamed_from.is_empty() {
                        db.execute(
                            TableAlterStatement::new()
                                .table(self.table.get_table_name().expect("Checked above").clone())
                                .add_column(column_def.to_owned()),
                        )?;
                    } else {
                        db.execute(
                            TableAlterStatement::new()
                                .table(self.table.get_table_name().expect("Checked above").clone())
                                .rename_column(
                                    renamed_from.to_owned(),
                                    column_def.get_column_name(),
                                ),
                        )?;
                    }
                }
            }
            if db.get_database_backend() != DbBackend::Sqlite {
                for foreign_key in self.table.get_foreign_key_create_stmts().iter() {
                    let mut key_exists = false;
                    for existing_key in existing_table.get_foreign_key_create_stmts().iter() {
                        if compare_foreign_key(foreign_key, existing_key) {
                            key_exists = true;
                            break;
                        }
                    }
                    if !key_exists {
                        db.execute(foreign_key)?;
                    }
                }
            }
        } else {
            db.execute(&self.table)?;
        }
        for stmt in self.indexes.iter() {
            let mut has_index = false;
            if let Some(existing_table) = existing_table {
                for existing_index in existing_table.get_indexes() {
                    if existing_index.get_index_spec().get_column_names()
                        == stmt.get_index_spec().get_column_names()
                    {
                        has_index = true;
                        break;
                    }
                }
            }
            if !has_index {
                // shall we do alter table add constraint for unique index?
                let mut stmt = stmt.clone();
                stmt.if_not_exists();
                db.execute(&stmt)?;
            }
        }
        if let Some(existing_table) = existing_table {
            // For columns with a column-level UNIQUE constraint (#[sea_orm(unique)]) that
            // already exist in the table but do not yet have a unique index, create one.
            for column_def in self.table.get_columns() {
                if column_def.get_column_spec().unique {
                    let col_name = column_def.get_column_name();
                    let col_exists = existing_table
                        .get_columns()
                        .iter()
                        .any(|c| c.get_column_name() == col_name);
                    if !col_exists {
                        // Column is being added in this sync pass; the ALTER TABLE ADD COLUMN
                        // will include the UNIQUE inline, so no separate index needed.
                        continue;
                    }
                    let already_unique = existing_table.get_indexes().iter().any(|idx| {
                        if !idx.is_unique_key() {
                            return false;
                        }
                        let cols = idx.get_index_spec().get_column_names();
                        cols.len() == 1 && cols[0] == col_name
                    });
                    if !already_unique {
                        let table_name =
                            self.table.get_table_name().expect("table must have a name");
                        let tbl_str = table_name.sea_orm_table().to_string();
                        let table_ref = table_name.clone();
                        db.execute(
                            Index::create()
                                .name(format!("idx-{tbl_str}-{col_name}"))
                                .table(table_ref)
                                .col(col_name.into_iden())
                                .unique()
                                .if_not_exists(),
                        )?;
                    }
                }
            }
        }
        if let Some(existing_table) = existing_table {
            // find all unique keys from existing table
            // if it no longer exist in new schema, drop it
            for existing_index in existing_table.get_indexes() {
                if existing_index.is_unique_key() {
                    let mut has_index = false;
                    for stmt in self.indexes.iter() {
                        if existing_index.get_index_spec().get_column_names()
                            == stmt.get_index_spec().get_column_names()
                        {
                            has_index = true;
                            break;
                        }
                    }
                    // Also check if the unique index corresponds to a column-level UNIQUE
                    // constraint (from #[sea_orm(unique)]). These are embedded in the CREATE
                    // TABLE column definition and not tracked in self.indexes, so we must not
                    // try to drop them during sync.
                    if !has_index {
                        let index_cols = existing_index.get_index_spec().get_column_names();
                        if index_cols.len() == 1 {
                            for column_def in self.table.get_columns() {
                                if column_def.get_column_name() == index_cols[0]
                                    && column_def.get_column_spec().unique
                                {
                                    has_index = true;
                                    break;
                                }
                            }
                        }
                    }
                    if !has_index {
                        if let Some(drop_existing) = existing_index.get_index_spec().get_name() {
                            db.execute(sea_query::Index::drop().name(drop_existing))?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn debug_print(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        backend: &DbBackend,
    ) -> std::fmt::Result {
        write!(f, "EntitySchemaInfo {{")?;
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

fn compare_foreign_key(a: &ForeignKeyCreateStatement, b: &ForeignKeyCreateStatement) -> bool {
    let a = a.get_foreign_key();
    let b = b.get_foreign_key();

    a.get_name() == b.get_name()
        || (a.get_ref_table() == b.get_ref_table()
            && a.get_columns() == b.get_columns()
            && a.get_ref_columns() == b.get_ref_columns())
}
