//! Phase 2: Interpret recorded schema changes into SQL statements, warnings, and suggestions.
//!
//! The main entry point is [`interpret`], which takes a [`ChangeSet`] from Phase 1
//! and produces an [`InterpretResult`] containing SQL statements, warnings,
//! suggestions, and unresolved ambiguous renames.

use std::collections::{HashMap, HashSet};

use sea_query::TableAlterStatement;

use super::changes::{
    ChangeId, ChangeSet, ColumnChange, ColumnChangeKind, ConstraintChange, ConstraintChangeKind,
    EnumChange, EnumChangeKind, SchemaChange, TableChange, TableChangeKind,
};
use super::resolver::{self, AddedColumn, RemovedColumn};
use super::suggestion::{DiscoverSuggestion, SuggestionKind};
use super::warning::{DiscoverWarning, WarningKind};
use crate::{DbBackend, Statement};

/// Result of interpreting recorded schema changes (Phase 2).
#[cfg_attr(docsrs, doc(cfg(feature = "schema-sync")))]
#[derive(Debug, Default)]
pub struct InterpretResult {
    /// SQL statements needed to bring the database in sync with entity definitions.
    /// Each entry is paired with the [`ChangeId`] it was generated from.
    pub statements: Vec<(ChangeId, Statement)>,
    /// Always-on warnings about changes requiring manual attention (e.g. data migration).
    pub warnings: Vec<DiscoverWarning>,
    /// Heuristic-powered suggested fixes (renames, enum changes).
    pub suggestions: Vec<DiscoverSuggestion>,
    /// Ambiguous renames that need user input to resolve.
    pub unresolved: Vec<resolver::AmbiguousRename>,
    /// Renames that were auto-applied because they were an obvious 1:1 match
    /// (see [`resolver::RenameResolution::assumed`]). Each entry names the
    /// statement in `statements` it produced, plus the DROP + ADD pair that
    /// should replace it if the caller rejects the assumption.
    pub assumed: Vec<AssumedRename>,
}

impl InterpretResult {
    /// Get just the SQL statements (without change IDs).
    pub fn sql_statements(&self) -> Vec<&Statement> {
        self.statements.iter().map(|(_, s)| s).collect()
    }

    /// Reject an auto-applied rename: remove its RENAME COLUMN statement and
    /// replace it with the separate DROP COLUMN + ADD COLUMN it was assumed from.
    /// No-op if `id` does not match any entry in `assumed`.
    pub fn reject_assumed(&mut self, id: ChangeId) {
        let Some(pos) = self.assumed.iter().position(|a| a.id == id) else {
            return;
        };
        let assumed = self.assumed.remove(pos);
        self.statements.retain(|(sid, _)| *sid != id);
        self.statements
            .push((assumed.drop_id, assumed.fallback_drop));
        self.statements.push((assumed.add_id, assumed.fallback_add));
    }

    /// Exclude a set of statements from the result entirely, identified by [`ChangeId`].
    pub fn exclude(&mut self, ids: &HashSet<ChangeId>) {
        self.statements.retain(|(sid, _)| !ids.contains(sid));
    }
}

/// A rename that was auto-applied because the resolver considered it an
/// obvious 1:1 match (same type, close proximity). Carries the fallback
/// DROP + ADD statements needed if the caller rejects the assumption via
/// [`InterpretResult::reject_assumed`].
#[cfg_attr(docsrs, doc(cfg(feature = "schema-sync")))]
#[derive(Debug, Clone)]
pub struct AssumedRename {
    /// The table the renamed column belongs to.
    pub table: String,
    /// The old (removed) column name.
    pub from: String,
    /// The new (added) column name.
    pub to: String,
    /// [`ChangeId`] of the RENAME COLUMN statement currently in `statements`.
    pub id: ChangeId,
    /// [`ChangeId`] to use for the fallback DROP COLUMN statement.
    pub drop_id: ChangeId,
    /// [`ChangeId`] to use for the fallback ADD COLUMN statement.
    pub add_id: ChangeId,
    fallback_drop: Statement,
    fallback_add: Statement,
}

/// A decision made about an ambiguous rename.
#[cfg_attr(docsrs, doc(cfg(feature = "schema-sync")))]
#[derive(Debug, Clone)]
pub enum RenameDecision {
    /// The user confirmed this is a rename.
    Rename {
        /// The old (removed) column name.
        from: String,
        /// The new (added) column name.
        to: String,
    },
    /// The user said this is not a rename — DROP + ADD.
    DropAndAdd {
        /// The removed column name.
        removed: String,
        /// The added column names that were candidates.
        added: Vec<String>,
    },
}

impl InterpretResult {
    /// Apply user decisions for ambiguous renames.
    pub fn apply_rename_decisions(&mut self, decisions: &[RenameDecision], db_backend: DbBackend) {
        for decision in decisions {
            match decision {
                RenameDecision::Rename { from, to } => {
                    if let Some(ambiguous) = self
                        .unresolved
                        .iter()
                        .find(|a| a.removed == *from && a.candidates.iter().any(|c| c.added == *to))
                    {
                        let table_name = &ambiguous.table;
                        let id = ChangeId(usize::MAX);
                        self.statements.push((
                            id,
                            db_backend.build(
                                TableAlterStatement::new()
                                    .table(sea_query::Alias::new(table_name.as_str()))
                                    .rename_column(from.clone(), to.clone()),
                            ),
                        ));
                    }
                }
                RenameDecision::DropAndAdd { removed, .. } => {
                    if let Some(ambiguous) = self.unresolved.iter().find(|a| a.removed == *removed)
                    {
                        let table_name = &ambiguous.table;
                        let id = ChangeId(usize::MAX);
                        self.statements.push((
                            id,
                            db_backend.build(
                                TableAlterStatement::new()
                                    .table(sea_query::Alias::new(table_name.as_str()))
                                    .drop_column(sea_query::Alias::new(removed.as_str())),
                            ),
                        ));
                    }
                }
            }
        }
        self.unresolved.clear();
    }
}

/// Configures how change interpretation is performed.
#[derive(Debug)]
pub struct InterpretConfig {
    /// The database backend to use for building SQL statements (for renames resolved at interpret time).
    pub db_backend: DbBackend,
    /// Whether to auto-apply heuristic renames as SQL changes.
    pub assumptions: bool,
    /// Whether dangerous operations (drops) are allowed.
    pub allow_dangerous: bool,
}

/// Phase 2: Interpret recorded changes into SQL statements, warnings, and suggestions.
///
/// Operates only on the [`ChangeSet`] from Phase 1. Changes carry pre-built
/// [`Statement`]s; interpretation decides which to emit and generates warnings/suggestions.
pub fn interpret(change_set: ChangeSet, config: &InterpretConfig) -> InterpretResult {
    let mut statements: Vec<(ChangeId, Statement)> = Vec::new();
    let mut warnings: Vec<DiscoverWarning> = Vec::new();
    let mut suggestions: Vec<DiscoverSuggestion> = Vec::new();
    let mut unresolved: Vec<resolver::AmbiguousRename> = Vec::new();
    let mut assumed: Vec<AssumedRename> = Vec::new();

    // Ordered to satisfy FK / type constraints:
    // 1. CREATE SCHEMA — namespaces must exist before anything created inside them
    // 2. CREATE TYPE  — enum types must exist before tables that reference them
    // 3. CREATE TABLE — parents before children (ChangeSet records in sorted_tables order)
    // 4. ADD COLUMN
    // 5. ADD FK / ADD INDEX / ADD UNIQUE
    // 6. DROP FK / DROP UNIQUE
    // 7. DROP COLUMN
    // 8. DROP TABLE   — children before parents (ChangeSet records via sorted_table_drops)
    // 9. DROP TYPE    — after tables that referenced the type are gone
    interpret_schema_creates(&change_set.schemas, &mut statements);
    interpret_enum_creates(&change_set.enums, &mut statements);
    interpret_table_creates(&change_set.tables, &mut statements);
    interpret_column_adds(
        &change_set.columns,
        config,
        &mut statements,
        &mut warnings,
        &mut suggestions,
        &mut unresolved,
        &mut assumed,
    );
    interpret_constraint_adds(&change_set.constraints, &mut statements);
    interpret_constraint_drops(&change_set.constraints, &mut statements);
    interpret_column_drops(&change_set.columns, config, &mut statements);
    interpret_table_drops(&change_set.tables, config, &mut statements);
    interpret_enum_drops(&change_set.enums, config, &mut statements, &mut suggestions);

    InterpretResult {
        statements,
        warnings,
        suggestions,
        unresolved,
        assumed,
    }
}

/// Emit CREATE SCHEMA statements — always first, so nothing created inside a
/// namespace can run before the namespace itself exists.
fn interpret_schema_creates(schemas: &[SchemaChange], statements: &mut Vec<(ChangeId, Statement)>) {
    for sc in schemas {
        statements.push((sc.id, sc.stmt.clone()));
    }
}

/// Emit CREATE TABLE statements (parents before children via ChangeSet recording order).
fn interpret_table_creates(tables: &[TableChange], statements: &mut Vec<(ChangeId, Statement)>) {
    for tc in tables {
        if let TableChangeKind::Create { stmt, .. } = &tc.kind {
            statements.push((tc.id, stmt.clone()));
        }
    }
}

/// Emit DROP TABLE statements (children before parents via ChangeSet recording order).
fn interpret_table_drops(
    tables: &[TableChange],
    config: &InterpretConfig,
    statements: &mut Vec<(ChangeId, Statement)>,
) {
    for tc in tables {
        if let TableChangeKind::Drop { table } = &tc.kind {
            statements.push((
                tc.id,
                config.db_backend.build(
                    sea_query::Table::drop()
                        .table(sea_query::TableRef::Table(table.clone(), None))
                        .if_exists(),
                ),
            ));
        }
    }
}

/// Emit ADD COLUMN and RENAME COLUMN statements.
/// Also populates warnings, suggestions, and unresolved renames.
/// Drop statements are collected separately by `interpret_column_drops`.
fn interpret_column_adds(
    columns: &[ColumnChange],
    config: &InterpretConfig,
    statements: &mut Vec<(ChangeId, Statement)>,
    warnings: &mut Vec<DiscoverWarning>,
    suggestions: &mut Vec<DiscoverSuggestion>,
    unresolved: &mut Vec<resolver::AmbiguousRename>,
    assumed: &mut Vec<AssumedRename>,
) {
    let mut drop_stmts: Vec<(ChangeId, Statement)> = Vec::new();
    interpret_columns_inner(
        columns,
        config,
        statements,
        &mut drop_stmts,
        warnings,
        suggestions,
        unresolved,
        assumed,
    );
    // drop_stmts are discarded here; they will be emitted by interpret_column_drops
}

/// Emit DROP COLUMN statements (after FK drops, before table drops).
fn interpret_column_drops(
    columns: &[ColumnChange],
    config: &InterpretConfig,
    statements: &mut Vec<(ChangeId, Statement)>,
) {
    let mut add_stmts: Vec<(ChangeId, Statement)> = Vec::new();
    let mut drop_stmts: Vec<(ChangeId, Statement)> = Vec::new();
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();
    let mut unresolved = Vec::new();
    let mut assumed = Vec::new();
    interpret_columns_inner(
        columns,
        config,
        &mut add_stmts,
        &mut drop_stmts,
        &mut warnings,
        &mut suggestions,
        &mut unresolved,
        &mut assumed,
    );
    statements.extend(drop_stmts);
}

/// Core column interpretation: runs rename detection and separates ADD/RENAME from DROP outputs.
#[allow(clippy::too_many_arguments)]
fn interpret_columns_inner(
    columns: &[ColumnChange],
    config: &InterpretConfig,
    add_stmts: &mut Vec<(ChangeId, Statement)>,
    drop_stmts: &mut Vec<(ChangeId, Statement)>,
    warnings: &mut Vec<DiscoverWarning>,
    suggestions: &mut Vec<DiscoverSuggestion>,
    unresolved: &mut Vec<resolver::AmbiguousRename>,
    assumed: &mut Vec<AssumedRename>,
) {
    let mut table_added: HashMap<String, Vec<(ChangeId, AddedColumn, Statement)>> =
        Default::default();
    let mut table_removed: HashMap<String, Vec<(ChangeId, RemovedColumn, Statement)>> =
        Default::default();

    for cc in columns {
        match &cc.kind {
            ColumnChangeKind::Add {
                column,
                index,
                column_type,
                is_not_null,
                has_default,
                stmt,
            } => {
                if *is_not_null && !has_default {
                    warnings.push(DiscoverWarning {
                        kind: WarningKind::NotNullNoDefault,
                        message: format!(
                            "Column '{}.{column}' is NOT NULL with no default value. \
                             Existing rows will need data populated before or during this migration.",
                            cc.table,
                        ),
                        related_changes: vec![cc.id],
                    });
                }
                table_added.entry(cc.table.clone()).or_default().push((
                    cc.id,
                    AddedColumn {
                        index: *index,
                        name: column.clone(),
                        column_type: column_type.clone(),
                    },
                    stmt.clone(),
                ));
            }
            ColumnChangeKind::Drop {
                column,
                index,
                column_type,
                stmt,
            } => {
                table_removed.entry(cc.table.clone()).or_default().push((
                    cc.id,
                    RemovedColumn {
                        index: *index,
                        name: column.clone(),
                        column_type: column_type.clone(),
                    },
                    stmt.clone(),
                ));
            }
            ColumnChangeKind::ExplicitRename { from, to, stmt } => {
                if config.assumptions {
                    add_stmts.push((cc.id, stmt.clone()));
                } else {
                    suggestions.push(DiscoverSuggestion {
                        kind: SuggestionKind::PossibleRename,
                        message: format!(
                            "Column '{}.{from}' has a `renamed_from` annotation to '{to}'. \
                             Enable assumptions to auto-apply.",
                            cc.table,
                        ),
                        related_changes: vec![cc.id],
                    });
                }
            }
            ColumnChangeKind::CheckConstraintPresent { column } => {
                warnings.push(DiscoverWarning {
                    kind: WarningKind::CheckConstraintDiff,
                    message: format!(
                        "Column '{}.{column}' has a CHECK constraint in entity definition. \
                         CHECK constraints cannot be automatically diffed — verify manually.",
                        cc.table,
                    ),
                    related_changes: vec![cc.id],
                });
            }
        }
    }

    // Rename detection per table
    let all_tables: HashSet<String> = table_added
        .keys()
        .chain(table_removed.keys())
        .cloned()
        .collect();

    for table in &all_tables {
        let added = table_added.remove(table.as_str()).unwrap_or_default();
        let removed = table_removed.remove(table.as_str()).unwrap_or_default();

        if !config.allow_dangerous || (added.is_empty() && removed.is_empty()) {
            for (id, _, stmt) in &added {
                add_stmts.push((*id, stmt.clone()));
            }
            continue;
        }

        let added_ids: HashMap<String, ChangeId> = added
            .iter()
            .map(|(id, c, _)| (c.name.clone(), *id))
            .collect();
        let removed_ids: HashMap<String, ChangeId> = removed
            .iter()
            .map(|(id, c, _)| (c.name.clone(), *id))
            .collect();
        let added_stmts: HashMap<String, Statement> = added
            .iter()
            .map(|(_, c, s)| (c.name.clone(), s.clone()))
            .collect();
        let removed_stmts: HashMap<String, Statement> = removed
            .iter()
            .map(|(_, c, s)| (c.name.clone(), s.clone()))
            .collect();

        let resolver_added: Vec<AddedColumn> = added.into_iter().map(|(_, c, _)| c).collect();
        let resolver_removed: Vec<RemovedColumn> = removed.into_iter().map(|(_, c, _)| c).collect();

        let resolution = resolver::resolve_renames(table, resolver_added, resolver_removed);

        // Assumed renames
        for rename in &resolution.assumed {
            let add_id = added_ids[&rename.added];
            let drop_id = removed_ids[&rename.removed];

            if config.assumptions {
                add_stmts.push((
                    add_id,
                    config.db_backend.build(
                        TableAlterStatement::new()
                            .table(sea_query::Alias::new(table.as_str()))
                            .rename_column(rename.removed.clone(), rename.added.clone()),
                    ),
                ));
                suggestions.push(DiscoverSuggestion {
                    kind: SuggestionKind::PossibleRename,
                    message: format!(
                        "Column '{table}.{}' was auto-renamed to '{}' \
                         (same type, position proximity {}). Use `--rename` to override.",
                        rename.removed, rename.added, rename.proximity,
                    ),
                    related_changes: vec![add_id, drop_id],
                });
                assumed.push(AssumedRename {
                    table: table.clone(),
                    from: rename.removed.clone(),
                    to: rename.added.clone(),
                    id: add_id,
                    drop_id,
                    add_id,
                    fallback_drop: removed_stmts[&rename.removed].clone(),
                    fallback_add: added_stmts[&rename.added].clone(),
                });
            } else {
                suggestions.push(DiscoverSuggestion {
                    kind: SuggestionKind::PossibleRename,
                    message: format!(
                        "Column '{table}.{}' may have been renamed to '{}' \
                         (same type, position proximity {}). Enable assumptions or use `--rename` to apply.",
                        rename.removed, rename.added, rename.proximity,
                    ),
                    related_changes: vec![add_id, drop_id],
                });
                add_stmts.push((add_id, added_stmts[&rename.added].clone()));
                drop_stmts.push((drop_id, removed_stmts[&rename.removed].clone()));
            }
        }

        unresolved.extend(resolution.ambiguous);

        for add in &resolution.remaining_added {
            let id = added_ids[&add.name];
            add_stmts.push((id, added_stmts[&add.name].clone()));
        }

        for rem in &resolution.remaining_removed {
            let id = removed_ids[&rem.name];
            drop_stmts.push((id, removed_stmts[&rem.name].clone()));
        }
    }
}

/// Emit ADD FOREIGN KEY, ADD INDEX, ADD UNIQUE CONSTRAINT statements.
fn interpret_constraint_adds(
    constraints: &[ConstraintChange],
    statements: &mut Vec<(ChangeId, Statement)>,
) {
    for cc in constraints {
        match &cc.kind {
            ConstraintChangeKind::AddForeignKey { stmt }
            | ConstraintChangeKind::AddIndex { stmt }
            | ConstraintChangeKind::AddUniqueConstraint { stmt, .. } => {
                statements.push((cc.id, stmt.clone()));
            }
            ConstraintChangeKind::DropForeignKey { .. }
            | ConstraintChangeKind::DropUniqueConstraint { .. } => {}
        }
    }
}

/// Emit DROP FOREIGN KEY and DROP UNIQUE CONSTRAINT statements (before column/table drops).
fn interpret_constraint_drops(
    constraints: &[ConstraintChange],
    statements: &mut Vec<(ChangeId, Statement)>,
) {
    for cc in constraints {
        match &cc.kind {
            ConstraintChangeKind::DropForeignKey { stmt, .. }
            | ConstraintChangeKind::DropUniqueConstraint { stmt, .. } => {
                statements.push((cc.id, stmt.clone()));
            }
            ConstraintChangeKind::AddForeignKey { .. }
            | ConstraintChangeKind::AddIndex { .. }
            | ConstraintChangeKind::AddUniqueConstraint { .. } => {}
        }
    }
}

/// Emit CREATE TYPE statements and variant-change/rename suggestions.
fn interpret_enum_creates(enums: &[EnumChange], statements: &mut Vec<(ChangeId, Statement)>) {
    for ec in enums {
        if let EnumChangeKind::Create { stmt } = &ec.kind {
            statements.push((ec.id, stmt.clone()));
        }
    }
}

/// Emit variant-change / rename suggestions, and DROP TYPE when allow_dangerous.
/// Must run after table drops so the enum is no longer referenced.
fn interpret_enum_drops(
    enums: &[EnumChange],
    config: &InterpretConfig,
    statements: &mut Vec<(ChangeId, Statement)>,
    suggestions: &mut Vec<DiscoverSuggestion>,
) {
    for ec in enums {
        match &ec.kind {
            EnumChangeKind::VariantChange { name, .. } => {
                if config.allow_dangerous {
                    suggestions.push(DiscoverSuggestion {
                        kind: SuggestionKind::EnumVariantChange,
                        message: format!(
                            "Enum type '{name}' has changed variants. Adding variants requires \
                             `ALTER TYPE ... ADD VALUE`; removing variants requires type recreation. \
                             This migration must be written manually.",
                        ),
                        related_changes: vec![ec.id],
                    });
                }
            }
            EnumChangeKind::Rename {
                existing_name,
                new_name,
            } => {
                if config.allow_dangerous {
                    suggestions.push(DiscoverSuggestion {
                        kind: SuggestionKind::EnumRename,
                        message: format!(
                            "Enum type '{existing_name}' appears to have been renamed to '{new_name}'. \
                             This requires `ALTER TYPE ... RENAME TO`.",
                        ),
                        related_changes: vec![ec.id],
                    });
                }
            }
            EnumChangeKind::Drop { stmt, .. } => {
                if config.allow_dangerous {
                    statements.push((ec.id, stmt.clone()));
                }
            }
            EnumChangeKind::Create { .. } => {}
        }
    }
}
