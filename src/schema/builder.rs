use super::{Schema, TopologicalSort};
use crate::{ConnectionTrait, DbBackend, DbErr, EntityTrait, Statement};
use sea_query::{
    IndexCreateStatement, TableCreateStatement, TableName, TableRef,
    extension::postgres::TypeCreateStatement,
};

#[cfg(feature = "schema-sync")]
pub use super::discover::resolver::extract_enum_type_name;
#[cfg(feature = "schema-sync")]
pub use super::discover::{
    DiscoverSuggestion, DiscoverWarning, InterpretConfig, InterpretResult, RenameDecision,
    SchemaChangeId, SuggestionKind, WarningKind, interpret::interpret as interpret_changes,
};

/// A schema builder that can take a registry of Entities and synchronize it with database.
pub struct SchemaBuilder {
    helper: Schema,
    entities: Vec<EntitySchemaInfo>,
    #[cfg(feature = "schema-sync")]
    excluded_tables: Vec<String>,
}

/// Schema info for Entity. Can be used to re-create schema in database.
pub struct EntitySchemaInfo {
    table: TableCreateStatement,
    enums: Vec<TypeCreateStatement>,
    indexes: Vec<IndexCreateStatement>,
    /// The schema name from the entity definition (e.g., `#[sea_orm(schema_name = "sys")]`).
    /// `None` means the entity uses the database's current/default schema.
    schema_name: Option<String>,
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
            #[cfg(feature = "schema-sync")]
            excluded_tables: Default::default(),
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

    /// Exclude tables from schema discovery.
    ///
    /// Excluded tables are never reported as orphans and are never diffed for column/FK changes.
    /// Use this to protect system tables (e.g. the migration tracker) from being dropped.
    #[cfg(feature = "schema-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "schema-sync")))]
    pub fn exclude(mut self, table: impl Into<String>) -> Self {
        self.excluded_tables.push(table.into());
        self
    }

    /// Synchronize the schema with the database: creates any missing tables, columns,
    /// unique keys, and foreign keys.
    ///
    /// Non-destructive by design. Sync only adds — it never drops or alters existing tables
    /// or columns. If a column already exists but its type or constraints differ from the
    /// entity, sync leaves it untouched and logs a warning; apply such changes with a
    /// migration. Destructive operations (ALTER / DROP) are intentionally out of scope and
    /// would be a separate, explicitly-named API.
    ///
    /// Unstable: schema sync is experimental and exempt from semver — its behaviour and
    /// signature may change in a minor (2.x) release.
    #[cfg(feature = "schema-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "schema-sync")))]
    pub async fn sync<C>(self, db: &C) -> Result<(), DbErr>
    where
        C: ConnectionTrait + sea_schema::Connection,
    {
        let change_set = self.discover(db, true).await?;
        for stmt in change_set.statements() {
            db.execute_raw(stmt).await?;
        }
        Ok(())
    }

    /// Returns a [`ChangeSet`](super::discover::changes::ChangeSet) grouped by origin.
    /// Use [`interpret`](super::discover::interpret) to turn it into SQL statements.
    ///
    /// * `db` - The database connection to use for fetching existing table schema.
    /// * `allow_dangerous` - If `true`, changes will include drops (tables, columns, constraints).
    ///
    /// Panics if TableCreateStatement any table name is empty, will never happen.
    #[cfg(feature = "schema-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "schema-sync")))]
    pub async fn discover<C>(
        &self,
        db: &C,
        allow_dangerous: bool,
    ) -> Result<super::discover::changes::ChangeSet, DbErr>
    where
        C: ConnectionTrait + sea_schema::Connection,
    {
        super::discover::discover(&self.entities, db, allow_dangerous, &self.excluded_tables).await
    }

    /// Returns the SQL DDL statements (CREATE TABLE, CREATE TYPE, CREATE INDEX) for all
    /// registered entities, rendered for the builder's backend.
    ///
    /// Tables are ordered topologically (parents before children). Useful for previewing
    /// the schema without connecting to a database.
    pub fn schema_statements(&self) -> Vec<Statement> {
        let backend = self.helper.backend;
        let mut stmts: Vec<Statement> = Vec::new();
        let table_refs: Vec<&TableCreateStatement> =
            self.entities.iter().map(|e| &e.table).collect();
        for table_name in sorted_tables(&table_refs, TableSortOrder::ParentsFirst) {
            if let Some(entity) = self
                .entities
                .iter()
                .find(|e| table_name == get_table_name(e.table.get_table_name()))
            {
                for stmt in &entity.enums {
                    stmts.push(backend.build(stmt));
                }
                stmts.push(backend.build(&entity.table));
                for stmt in &entity.indexes {
                    stmts.push(backend.build(stmt));
                }
            }
        }
        stmts
    }

    /// Create all registered tables, columns, unique keys, and foreign keys.
    /// Fails if any table already exists. Use `sync` (feature `schema-sync`)
    /// instead for an incremental version that diffs against the live schema.
    pub async fn apply<C: ConnectionTrait>(self, db: &C) -> Result<(), DbErr> {
        let mut created_enums: Vec<Statement> = Default::default();

        let table_refs: Vec<&TableCreateStatement> =
            self.entities.iter().map(|entity| &entity.table).collect();
        for table_name in sorted_tables(&table_refs, TableSortOrder::ParentsFirst) {
            if let Some(entity) = self
                .entities
                .iter()
                .find(|entity| table_name == get_table_name(entity.table.get_table_name()))
            {
                entity.apply(db, &mut created_enums).await?;
            }
        }

        Ok(())
    }

    // Regression guard for #3100: `sync()` must return a `Send` future, otherwise it
    // cannot be used from `tokio::spawn` / most async runtimes. Compiled (never called)
    // whenever `schema-sync` + a backend is on, so a future dep bump that reintroduces a
    // `!Send` value across an await fails the build here.
    #[allow(dead_code)]
    #[cfg(all(feature = "schema-sync", feature = "sqlx-sqlite"))]
    fn _assert_sync_future_is_send(self, db: &crate::DatabaseConnection) {
        fn assert_send<T: Send>(_: &T) {}
        assert_send(&self.sync(db));
    }
}

impl EntitySchemaInfo {
    /// Creates a EntitySchemaInfo object given a generic Entity.
    pub fn new<E: EntityTrait>(entity: E, helper: &Schema) -> Self {
        Self {
            table: helper.create_table_from_entity(entity),
            enums: helper.create_enum_from_entity(entity),
            indexes: helper.create_index_from_entity(entity),
            schema_name: entity.schema_name().map(|s| s.to_string()),
        }
    }

    /// Returns a reference to the table create statement.
    #[cfg(feature = "schema-sync")]
    pub(crate) fn table(&self) -> &TableCreateStatement {
        &self.table
    }

    /// Returns a reference to the enum type create statements.
    #[cfg(feature = "schema-sync")]
    pub(crate) fn enums(&self) -> &[TypeCreateStatement] {
        &self.enums
    }

    /// Returns a reference to the index create statements.
    #[cfg(feature = "schema-sync")]
    pub(crate) fn indexes(&self) -> &[IndexCreateStatement] {
        &self.indexes
    }

    /// Returns the entity's `schema_name` (e.g. `#[sea_orm(schema_name = "sys")]`).
    /// `None` means the entity uses the database's current/default schema
    #[cfg(feature = "schema-sync")]
    pub(crate) fn schema_name(&self) -> Option<&str> {
        self.schema_name.as_deref()
    }

    async fn apply<C: ConnectionTrait>(
        &self,
        db: &C,
        created_enums: &mut Vec<Statement>,
    ) -> Result<(), DbErr> {
        for stmt in self.enums.iter() {
            let new_stmt = db.get_database_backend().build(stmt);
            if !created_enums.iter().any(|s| s == &new_stmt) {
                db.execute(stmt).await?;
                created_enums.push(new_stmt);
            }
        }
        db.execute(&self.table).await?;
        for stmt in self.indexes.iter() {
            db.execute(stmt).await?;
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

/// Panics if the table reference is not a table name
pub(crate) fn get_table_name(table_ref: Option<&TableRef>) -> TableName {
    //TODO: either rewrite TableCreateStatement or move to something else that is not a builder with options
    match table_ref {
        Some(TableRef::Table(table_name, _)) => table_name.clone(),
        None => panic!("Expect TableCreateStatement is properly built"),
        _ => unreachable!("Unexpected {table_ref:?}"),
    }
}

/// Controls which tables appear first in [`sorted_tables`] output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TableSortOrder {
    /// Parent tables (no FK dependents) appear before children
    ParentsFirst,
    /// Child tables (FK holders) appear before parents
    ChildrenFirst,
}

/// Sort table names topologically by FK dependency
pub(crate) fn sorted_tables(
    tables: &[&TableCreateStatement],
    order: TableSortOrder,
) -> Vec<TableName> {
    let mut sorter = TopologicalSort::<TableName>::new();

    for tbl in tables {
        sorter.insert(get_table_name(tbl.get_table_name()));
    }
    for tbl in tables {
        let self_name = get_table_name(tbl.get_table_name());
        for fk in tbl.get_foreign_key_create_stmts() {
            let ref_table = get_table_name(fk.get_foreign_key().get_ref_table());
            if self_name != ref_table {
                match order {
                    TableSortOrder::ParentsFirst => {
                        sorter.add_dependency(ref_table.clone(), self_name.clone());
                    }
                    TableSortOrder::ChildrenFirst => {
                        sorter.add_dependency(self_name.clone(), ref_table.clone());
                    }
                }
            }
        }
    }
    let mut sorted = Vec::new();
    loop {
        // Collect all zero-predecessor nodes, sort by name for determinism,
        // then drain them one level at a time. Without this sort, HashMap
        // iteration order inside TopologicalSort::peek() is random per process,
        // causing different orderings across subprocess invocations (e.g. diff
        // vs generate in `entity sync`), which breaks the schema-hash check.
        let mut level = sorter.pop_all();
        if level.is_empty() {
            break;
        }
        level.sort_by(|a, b| a.1.to_string().cmp(&b.1.to_string()));
        sorted.extend(level);
    }

    // Append any leftovers (circular deps)
    for tbl in tables {
        let name = get_table_name(tbl.get_table_name());
        if !sorted.contains(&name) {
            sorted.push(name);
        }
    }
    sorted
}
