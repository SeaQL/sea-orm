pub mod changes;
mod enum_;
pub mod interpret;
pub mod resolver;
pub(crate) mod schema;
pub mod suggestion;
mod table;
pub mod warning;

use crate::schema::builder::{EntitySchemaInfo, TableSortOrder, table_id};
use crate::{ConnectionTrait, DbErr, TableId, sorted_tables};
use changes::ChangeSet;

pub use changes::ChangeId as SchemaChangeId;
pub use changes::ColumnSignature;
pub use interpret::{
    AssumedRename, AssumedTableMove, InterpretConfig, InterpretResult, RenameDecision,
};
use sea_query::TableCreateStatement;
pub use suggestion::{DiscoverSuggestion, SuggestionKind};
pub use warning::{DiscoverWarning, WarningKind};

//TODO: honestly, I think whole scheam module should be moved to a separate crate

/// Record all schema changes by comparing entities against the database
pub(crate) async fn discover<C>(
    new_entities: &[EntitySchemaInfo],
    db: &C,
    excluded_tables: &[String],
    excluded_schemas: &[String],
) -> Result<ChangeSet, DbErr>
where
    C: ConnectionTrait + sea_schema::Connection,
{
    let db_backend = db.get_database_backend();

    // Multi scheme
    let mut extra_schemas: Vec<String> = new_entities
        .iter()
        .filter_map(|e| e.schema_name().map(str::to_owned))
        .filter(|s| !excluded_schemas.iter().any(|e| e == s))
        .collect();

    // Orphan detection needs to see every schema, not just
    // ones a current entity references — otherwise a schema no entity
    // references anymore (e.g. after a `schema_name` rename) is invisible,
    // and its now-orphaned tables can never be reported.
    //
    // Postgres-only: a Postgres `schema_name` is a namespace inside the one
    // database this connection already owns, so scanning every non-system
    // schema in it is self-contained and safe. On MySQL, `schema_name`
    // addresses a separate *database* on the server — scanning "every schema"
    // there means scanning every database the connection can see, which is
    // very likely to include databases belonging to other applications/tests
    // sharing the server. There's no safe way to auto-discover which of those
    // used to be "ours", so MySQL orphan detection is intentionally limited
    // to databases a currently-registered entity's `schema_name` still points at.
    #[cfg(feature = "sqlx-postgres")]
    if db_backend == crate::DbBackend::Postgres {
        extra_schemas.extend(schema::pg_list_all_schemas(db, excluded_schemas).await?);
    }

    extra_schemas.sort_unstable();
    extra_schemas.dedup();

    let existing = schema::discover_existing_schema(db, &extra_schemas).await?;

    let mut change_set = ChangeSet::default();

    for missing_schema in &existing.missing_schemas {
        let stmt = crate::schema::builder::create_schema_stmt(db_backend, missing_schema);
        change_set.record_schema(missing_schema.clone(), stmt);
    }

    let tabl_ref: Vec<&TableCreateStatement> = new_entities.iter().map(|e| e.table()).collect();
    let entities_by_table: std::collections::HashMap<TableId, &EntitySchemaInfo> = new_entities
        .iter()
        .map(|e| (table_id(e.table()), e))
        .collect();
    for table in sorted_tables(&tabl_ref, TableSortOrder::ParentsFirst) {
        let table = table_id(table);
        if excluded_tables.iter().any(|e| e == &table.name) {
            continue;
        }

        let entity = entities_by_table[&table];
        enum_::record_enum_changes(entity.enums(), db_backend, &existing.enums, &mut change_set);
        table::record_table_changes(entity, &existing.tables, &mut change_set, db_backend);
    }

    table::record_orphan_tables(new_entities, &existing, &mut change_set, excluded_tables);
    let all_entity_enums: Vec<&sea_query::extension::postgres::TypeCreateStatement> =
        new_entities.iter().flat_map(|e| e.enums().iter()).collect();
    enum_::record_orphan_enums(
        &all_entity_enums,
        db_backend,
        &existing.enums,
        &mut change_set,
    );

    Ok(change_set)
}
