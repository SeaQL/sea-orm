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
) -> Result<ChangeSet, DbErr>
where
    C: ConnectionTrait + sea_schema::Connection,
{
    let existing = schema::discover_existing_schema(db).await?;
    let db_backend = db.get_database_backend();

    let mut change_set = ChangeSet::default();

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
