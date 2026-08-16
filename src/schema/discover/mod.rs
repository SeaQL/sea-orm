pub mod changes;
mod enum_;
pub mod interpret;
pub mod resolver;
pub(crate) mod schema;
pub mod suggestion;
mod table;
pub mod warning;

use crate::schema::builder::{EntitySchemaInfo, TableSortOrder, get_table_name};
use crate::{ConnectionTrait, DbErr, sorted_tables};
use changes::ChangeSet;

pub use changes::ChangeId as SchemaChangeId;
pub use interpret::{InterpretConfig, InterpretResult, RenameDecision};
use sea_query::TableCreateStatement;
pub use suggestion::{DiscoverSuggestion, SuggestionKind};
pub use warning::{DiscoverWarning, WarningKind};

//TODO: honestly, I think whole scheam module should be moved to a separate crate

/// Record all schema changes by comparing entities against the database
pub(crate) async fn discover<C>(
    new_entities: &[EntitySchemaInfo],
    db: &C,
    allow_dangerous: bool,
    excluded_tables: &[String],
    excluded_schemas: &[String],
) -> Result<ChangeSet, DbErr>
where
    C: ConnectionTrait + sea_schema::Connection,
{
    let db_backend = db.get_database_backend();
    // Only read by the Postgres/MySQL branches below.
    let _ = excluded_schemas;

    // Multi scheme
    let mut extra_schemas: Vec<String> = new_entities
        .iter()
        .filter_map(|e| e.schema_name().map(str::to_owned))
        .collect();

    // Orphan detection (allow_dangerous) needs to see every schema, not just
    // ones a current entity references — otherwise a schema no entity
    // references anymore (e.g. after a `schema_name` rename) is invisible,
    // and its now-orphaned tables can never be reported.
    #[cfg(feature = "sqlx-postgres")]
    if allow_dangerous && db_backend == crate::DbBackend::Postgres {
        extra_schemas.extend(schema::pg_list_all_schemas(db, excluded_schemas).await?);
    }
    #[cfg(feature = "sqlx-mysql")]
    if allow_dangerous && db_backend == crate::DbBackend::MySql {
        extra_schemas.extend(schema::mysql_list_all_schemas(db, excluded_schemas).await?);
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
    for table_name in sorted_tables(&tabl_ref, TableSortOrder::ParentsFirst) {
        let name_str = table_name.1.to_string();
        if excluded_tables.iter().any(|e| e == &name_str) {
            continue;
        }

        //PERF: just sort TableCreateStatements, instead of searching
        if let Some(entity) = new_entities
            .iter()
            .find(|entity| table_name == get_table_name(entity.table().get_table_name()))
        {
            enum_::record_enum_changes(
                entity.enums(),
                db_backend,
                &existing.enums,
                &mut change_set,
            );
            table::record_table_changes(
                entity,
                &existing.tables,
                &mut change_set,
                allow_dangerous,
                db_backend,
            );
        } else {
            unreachable!()
        }
    }

    if allow_dangerous {
        table::record_orphan_tables(new_entities, &existing, &mut change_set, excluded_tables);
        let all_entity_enums: Vec<&sea_query::extension::postgres::TypeCreateStatement> =
            new_entities.iter().flat_map(|e| e.enums().iter()).collect();
        enum_::record_orphan_enums(
            &all_entity_enums,
            db_backend,
            &existing.enums,
            &mut change_set,
        );
    }

    Ok(change_set)
}
