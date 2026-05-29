use super::changes::{ChangeSet, EnumChangeKind};
use super::resolver::{self, extract_enum_type_name};
use super::schema::DiscoveredSchema;
use crate::DbBackend;
use sea_query::extension::postgres::TypeCreateStatement;

/// Phase 1: Record enum types in the database that have no matching entity (allow_dangerous only).
/// Records a DROP TYPE for each orphan enum.
pub(crate) fn record_orphan_enums(
    all_entity_enums: &[&TypeCreateStatement],
    db_backend: DbBackend,
    existing: &[TypeCreateStatement],
    changes: &mut ChangeSet,
) {
    for existing_enum in existing {
        let existing_stmt = db_backend.build(existing_enum);
        let Some(existing_name) = extract_enum_type_name(&existing_stmt.sql) else {
            continue;
        };
        let in_entities = all_entity_enums.iter().any(|e| {
            let s = db_backend.build(*e);
            extract_enum_type_name(&s.sql).as_deref() == Some(existing_name.as_str())
        });
        if !in_entities {
            let stmt = db_backend.build(
                &sea_query::extension::postgres::Type::drop()
                    .name(sea_query::Alias::new(existing_name.as_str()))
                    .if_exists()
                    .to_owned(),
            );
            changes.record_enum(EnumChangeKind::Drop {
                name: existing_name,
                stmt,
            });
        }
    }
}

/// Phase 1: Record enum changes for a single entity's enum definitions against the existing schema.
pub(crate) fn record_enum_changes(
    entity_enums: &[TypeCreateStatement],
    db_backend: DbBackend,
    existing: &[TypeCreateStatement],
    changes: &mut ChangeSet,
) {
    for stmt in entity_enums.iter() {
        let new_stmt = db_backend.build(stmt);
        let new_sql = &new_stmt.sql;

        let mut exact_match = false;
        let mut change_detected = false;

        for existing_enum in existing {
            let existing_stmt = db_backend.build(existing_enum);
            if existing_stmt == new_stmt {
                exact_match = true;
                break;
            }
            if let Some(enum_change) = resolver::detect_enum_change(&existing_stmt.sql, new_sql) {
                change_detected = true;
                match enum_change {
                    resolver::EnumChange::VariantChange {
                        name,
                        existing_sql,
                        new_sql,
                    } => {
                        changes.record_enum(EnumChangeKind::VariantChange {
                            name,
                            existing_sql,
                            new_sql,
                        });
                    }
                    resolver::EnumChange::NameChange {
                        existing_name,
                        new_name,
                    } => {
                        changes.record_enum(EnumChangeKind::Rename {
                            existing_name,
                            new_name,
                        });
                    }
                }
                break;
            }
        }

        if !exact_match && !change_detected {
            let sql = new_sql.clone();
            changes.record_enum_create(&sql, new_stmt);
        }
    }
}
