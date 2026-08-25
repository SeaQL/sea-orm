use super::changes::{ChangeSet, ColumnChangeKind, ConstraintChangeKind, TableChangeKind};
use super::schema::DiscoveredSchema;
use crate::schema::builder::{EntitySchemaInfo, get_table_name};
use crate::schema::entity::index_table_ref;
use crate::{DbBackend, TableSortOrder, sorted_tables};
use sea_query::{ForeignKeyCreateStatement, Index, TableAlterStatement, TableCreateStatement};

/// Phase 1: Record table-level changes for a single entity against the existing schema.
pub(crate) fn record_table_changes(
    entity: &EntitySchemaInfo,
    existing: &[TableCreateStatement],
    changes: &mut ChangeSet,
    db_backend: DbBackend,
) {
    let table_name = get_table_name(entity.table().get_table_name());
    let table_name_str = table_name.1.to_string();
    let existing_table = existing
        .iter()
        .find(|tbl| get_table_name(tbl.get_table_name()) == table_name);

    if let Some(existing_table) = existing_table {
        record_column_changes(entity, existing_table, changes, db_backend);
        record_foreign_key_changes(entity, existing_table, &table_name_str, changes, db_backend);
        record_index_changes(entity, existing_table, &table_name_str, changes, db_backend);
        record_unique_constraint_changes(
            entity,
            existing_table,
            &table_name_str,
            changes,
            db_backend,
        );
        record_unique_constraint_drops(
            entity,
            existing_table,
            &table_name_str,
            changes,
            db_backend,
        );
    } else {
        changes.record_table(TableChangeKind::Create {
            table_ref: get_entity_table_name(entity),
            columns: column_signature(entity.table()),
            stmt: db_backend.build(entity.table()),
        });

        //TODO: move into its own record fn
        for stmt in entity.indexes() {
            let mut idx_stmt = stmt.clone();
            idx_stmt.if_not_exists();
            changes.record_constraint(
                table_name_str.clone(),
                ConstraintChangeKind::AddIndex {
                    stmt: db_backend.build(&idx_stmt),
                },
            );
        }
    }
}

/// Phase 1: Record tables in the database that have no matching entity.
/// Drops are recorded in reverse-dependency order (children first) to avoid FK violations.
pub(crate) fn record_orphan_tables(
    entities: &[EntitySchemaInfo],
    existing: &DiscoveredSchema,
    changes: &mut ChangeSet,
    excluded_tables: &[String],
) {
    let orphans: Vec<&TableCreateStatement> = existing
        .tables
        .iter()
        .filter(|tbl| {
            let name = get_table_name(tbl.get_table_name());
            let name_str = name.1.to_string();
            !excluded_tables.iter().any(|e| e == &name_str)
                && !entities
                    .iter()
                    .any(|e| get_table_name(e.table().get_table_name()) == name)
        })
        .collect();

    for tbl in sorted_tables(&orphans, TableSortOrder::ChildrenFirst) {
        changes.record_table(TableChangeKind::Drop {
            table: get_table_name(tbl.get_table_name()),
            columns: column_signature(tbl),
        });
    }
}

/// Column (name, type) signature in definition order — used to detect a
/// dropped table and a created table as the same table renamed/schema-moved.
fn column_signature(tbl: &TableCreateStatement) -> Vec<(String, Option<sea_query::ColumnType>)> {
    tbl.get_columns()
        .iter()
        .map(|c| {
            (
                c.get_column_name().to_string(),
                c.get_column_type().cloned(),
            )
        })
        .collect()
}

fn get_entity_table_name(entity: &EntitySchemaInfo) -> sea_query::TableRef {
    entity
        .table()
        .get_table_name()
        .expect("table must have a name")
        .clone()
}

fn record_column_changes(
    entity: &EntitySchemaInfo,
    existing_table: &sea_query::TableCreateStatement,
    changes: &mut ChangeSet,
    db_backend: DbBackend,
) {
    let entity_table_name = get_entity_table_name(entity);

    for (idx, column_def) in entity.table().get_columns().iter().enumerate() {
        let col_name = column_def.get_column_name();
        let exists_in_db = existing_table
            .get_columns()
            .iter()
            .any(|c| c.get_column_name() == col_name);

        if exists_in_db {
            if column_def.get_column_spec().check.is_some() {
                changes.record_column(
                    entity_table_name.clone(),
                    ColumnChangeKind::CheckConstraintPresent {
                        column: col_name.to_string(),
                    },
                );
            }
            if let Some(existing_col) = existing_table
                .get_columns()
                .iter()
                .find(|c| c.get_column_name() == col_name)
                && existing_col.get_column_type() != column_def.get_column_type()
            {
                changes.record_column(
                    entity_table_name.clone(),
                    ColumnChangeKind::TypeMismatch {
                        column: col_name.to_string(),
                        existing_type: existing_col.get_column_type().cloned(),
                        new_type: column_def.get_column_type().cloned(),
                    },
                );
            }
            continue;
        }

        // Check for explicit renamed_from annotation
        let mut renamed_from = "";
        if let Some(comment) = &column_def.get_column_spec().comment
            && let Some((_, suffix)) = comment.rsplit_once("renamed_from \"")
            && let Some((prefix, _)) = suffix.split_once('"')
        {
            renamed_from = prefix;
        }

        if !renamed_from.is_empty() {
            changes.record_column(
                entity_table_name.clone(),
                ColumnChangeKind::ExplicitRename {
                    from: renamed_from.to_string(),
                    to: col_name.to_string(),
                    stmt: db_backend.build(
                        TableAlterStatement::new()
                            .table(entity_table_name.clone())
                            .rename_column(renamed_from.to_string(), col_name.to_string()),
                    ),
                },
            );
        } else {
            let spec = column_def.get_column_spec();
            let is_not_null = matches!(spec.nullable, Some(false));
            changes.record_column(
                entity_table_name.clone(),
                ColumnChangeKind::Add {
                    column: col_name.to_string(),
                    index: idx,
                    column_type: column_def.get_column_type().cloned(),
                    is_not_null,
                    has_default: spec.default.is_some(),
                    stmt: db_backend.build(
                        TableAlterStatement::new()
                            .table(entity_table_name.clone())
                            .add_column(column_def.to_owned()),
                    ),
                },
            );
        }
    }

    // Removed columns (in DB but not in entity)
    let entity_table_name = get_entity_table_name(entity);
    for (idx, col) in existing_table.get_columns().iter().enumerate() {
        let col_name = col.get_column_name();
        let in_entity = entity
            .table()
            .get_columns()
            .iter()
            .any(|ec| ec.get_column_name() == col_name);
        if !in_entity {
            changes.record_column(
                entity_table_name.clone(),
                ColumnChangeKind::Drop {
                    column: col_name.to_string(),
                    index: idx,
                    column_type: col.get_column_type().cloned(),
                    stmt: db_backend.build(
                        TableAlterStatement::new()
                            .table(entity_table_name.clone())
                            .drop_column(sea_query::Alias::new(col_name)),
                    ),
                },
            );
        }
    }
}

fn record_foreign_key_changes(
    entity: &EntitySchemaInfo,
    existing_table: &sea_query::TableCreateStatement,
    table_name_str: &str,
    changes: &mut ChangeSet,
    db_backend: DbBackend,
) {
    // SQLite can only declare foreign keys inline in CREATE TABLE — sea-query
    // panics if asked to build a FK statement against an existing table
    // (see `ForeignKeyBuilder::prepare_foreign_key_create_statement`, always
    // `Mode::Alter`). Skip FK diffing there entirely, same as the old sync().
    if db_backend == DbBackend::Sqlite {
        return;
    }

    for foreign_key in entity.table().get_foreign_key_create_stmts().iter() {
        let key_exists = existing_table
            .get_foreign_key_create_stmts()
            .iter()
            .any(|existing_key| compare_foreign_key(foreign_key, existing_key));
        if !key_exists {
            changes.record_constraint(
                table_name_str.to_string(),
                ConstraintChangeKind::AddForeignKey {
                    stmt: db_backend.build(foreign_key),
                },
            );
        }
    }

    let entity_table_name = get_entity_table_name(entity);
    for existing_key in existing_table.get_foreign_key_create_stmts().iter() {
        let in_entity = entity
            .table()
            .get_foreign_key_create_stmts()
            .iter()
            .any(|fk| compare_foreign_key(fk, existing_key));
        if !in_entity {
            let fk = existing_key.get_foreign_key();
            if let Some(name) = fk.get_name() {
                changes.record_constraint(
                    table_name_str.to_string(),
                    ConstraintChangeKind::DropForeignKey {
                        name: name.to_owned(),
                        stmt: db_backend.build(
                            TableAlterStatement::new()
                                .table(entity_table_name.clone())
                                .drop_foreign_key(name.to_owned()),
                        ),
                    },
                );
            }
        }
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

fn record_index_changes(
    entity: &EntitySchemaInfo,
    existing_table: &sea_query::TableCreateStatement,
    table_name_str: &str,
    changes: &mut ChangeSet,
    db_backend: DbBackend,
) {
    for stmt in entity.indexes().iter() {
        let has_index = existing_table.get_indexes().iter().any(|existing_index| {
            existing_index.get_index_spec().get_column_names()
                == stmt.get_index_spec().get_column_names()
        });
        if !has_index {
            let mut idx_stmt = stmt.clone();
            idx_stmt.if_not_exists();
            changes.record_constraint(
                table_name_str.to_string(),
                ConstraintChangeKind::AddIndex {
                    stmt: db_backend.build(&idx_stmt),
                },
            );
        }
    }
}

fn record_unique_constraint_changes(
    entity: &EntitySchemaInfo,
    existing_table: &sea_query::TableCreateStatement,
    table_name_str: &str,
    changes: &mut ChangeSet,
    db_backend: DbBackend,
) {
    let entity_table_name = get_entity_table_name(entity);

    for column_def in entity.table().get_columns() {
        if column_def.get_column_spec().unique {
            let col_name = column_def.get_column_name();
            let col_exists = existing_table
                .get_columns()
                .iter()
                .any(|c| c.get_column_name() == col_name);
            if !col_exists {
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
                changes.record_constraint(
                    table_name_str.to_string(),
                    ConstraintChangeKind::AddUniqueConstraint {
                        column: col_name.to_string(),
                        stmt: db_backend.build(
                            Index::create()
                                .name(format!("idx-{table_name_str}-{col_name}"))
                                .table(index_table_ref(entity_table_name.clone(), db_backend))
                                .col(sea_query::Alias::new(col_name))
                                .unique()
                                .if_not_exists(),
                        ),
                    },
                );
            }
        }
    }
}

fn record_unique_constraint_drops(
    entity: &EntitySchemaInfo,
    existing_table: &sea_query::TableCreateStatement,
    table_name_str: &str,
    changes: &mut ChangeSet,
    db_backend: DbBackend,
) {
    let entity_table_name = get_entity_table_name(entity);

    for existing_index in existing_table.get_indexes() {
        if !existing_index.is_unique_key() {
            continue;
        }
        let mut has_index = entity.indexes().iter().any(|stmt| {
            existing_index.get_index_spec().get_column_names()
                == stmt.get_index_spec().get_column_names()
        });
        if !has_index {
            let index_cols = existing_index.get_index_spec().get_column_names();
            if index_cols.len() == 1 {
                has_index = entity.table().get_columns().iter().any(|column_def| {
                    column_def.get_column_name() == index_cols[0]
                        && column_def.get_column_spec().unique
                });
            }
        }
        if !has_index
            && let Some(name) = existing_index
                .get_index_spec()
                .get_name()
                .map(|s| s.to_owned())
        {
            let stmt = if db_backend == DbBackend::Postgres {
                db_backend.build(
                    TableAlterStatement::new()
                        .table(entity_table_name.clone())
                        .drop_constraint(name.clone()),
                )
            } else {
                db_backend.build(sea_query::Index::drop().name(name.clone()))
            };

            changes.record_constraint(
                table_name_str.to_string(),
                ConstraintChangeKind::DropUniqueConstraint { name, stmt },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::builder::EntitySchemaInfo;
    use crate::tests_cfg::fruit;
    use crate::{DbBackend, Schema};

    /// `fruit` has a FK (`cake_id`) missing from `existing_table`, so this
    /// records an `AddForeignKey`. On SQLite, sea-query panics when building
    /// any FK statement outside of `CREATE TABLE` (`Mode::Creation`) — see
    /// `ForeignKeyBuilder::prepare_foreign_key_create_statement`, which always
    /// builds in `Mode::Alter`. Regression test for that panic.
    #[test]
    fn record_foreign_key_changes_does_not_panic_on_sqlite() {
        let backend = DbBackend::Sqlite;
        let schema = Schema::new(backend);
        let entity = EntitySchemaInfo::new(fruit::Entity, &schema);
        let existing_table = sea_query::TableCreateStatement::new();

        let mut changes = ChangeSet::default();
        record_foreign_key_changes(&entity, &existing_table, "fruit", &mut changes, backend);

        // SQLite can't ALTER a FK onto an existing table, so nothing should
        // be recorded rather than a statement that would fail at execution.
        assert!(changes.constraints.is_empty());
    }
}
