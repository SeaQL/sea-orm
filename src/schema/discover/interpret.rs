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
    /// Table renames/schema-moves that were auto-applied because a dropped
    /// table and a created table had an identical column signature. Each
    /// entry names the statement(s) in `statements` it produced, plus the
    /// CREATE + DROP pair that should replace them if the caller rejects
    /// the assumption.
    pub table_moves: Vec<AssumedTableMove>,
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

    /// Reject an auto-applied table rename/schema-move: remove its statement(s)
    /// and replace them with the separate CREATE TABLE + DROP TABLE it was
    /// assumed from. No-op if `id` does not match any entry in `table_moves`.
    pub fn reject_table_move(&mut self, id: ChangeId) {
        let Some(pos) = self.table_moves.iter().position(|m| m.id == id) else {
            return;
        };
        let table_move = self.table_moves.remove(pos);
        self.statements
            .retain(|(sid, _)| *sid != table_move.id && *sid != table_move.drop_id);
        self.statements
            .push((table_move.id, table_move.fallback_create));
        self.statements
            .push((table_move.drop_id, table_move.fallback_drop));
    }
}

/// A rename that was auto-applied because the resolver considered it an
/// obvious 1:1 match (same type, close proximity). Carries the fallback
/// DROP + ADD statements needed if the caller rejects the assumption via
/// [`InterpretResult::reject_assumed`].
#[cfg_attr(docsrs, doc(cfg(feature = "schema-sync")))]
#[derive(Debug, Clone)]
pub struct AssumedRename {
    /// Full (possibly schema-qualified) reference to the table the renamed
    /// column belongs to.
    pub table_ref: sea_query::TableRef,
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

impl AssumedRename {
    /// The table the renamed column belongs to, qualified by schema when the
    /// table has one (e.g. `"my_schema.person"`).
    pub fn table_name(&self) -> String {
        resolver::qualified_table_name(&self.table_ref)
    }
}

/// A table rename and/or schema-move that was auto-applied because a dropped
/// table and a created table had an identical column signature (same names
/// and types, in the same order). Carries the fallback CREATE TABLE + DROP
/// TABLE statements needed if the caller rejects the assumption via
/// [`InterpretResult::reject_table_move`].
#[cfg_attr(docsrs, doc(cfg(feature = "schema-sync")))]
#[derive(Debug, Clone)]
pub struct AssumedTableMove {
    /// Full (possibly schema-qualified) reference to the table's old identity.
    pub from: sea_query::TableRef,
    /// Full (possibly schema-qualified) reference to the table's new identity.
    pub to: sea_query::TableRef,
    /// [`ChangeId`] of the (first) move statement currently in `statements`.
    pub id: ChangeId,
    /// [`ChangeId`] used for a second move statement when both the name and
    /// schema changed, and for the fallback DROP TABLE statement.
    pub drop_id: ChangeId,
    fallback_create: Statement,
    fallback_drop: Statement,
}

impl AssumedTableMove {
    /// The table's old (possibly schema-qualified) name, e.g. `"my_schema.person"`.
    pub fn from_name(&self) -> String {
        resolver::qualified_table_name(&self.from)
    }

    /// The table's new (possibly schema-qualified) name, e.g. `"my_schema.person"`.
    pub fn to_name(&self) -> String {
        resolver::qualified_table_name(&self.to)
    }
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
        // IDs must be unique so callers can address each applied decision
        // individually (e.g. via `exclude`) — start above every ID already
        // in use anywhere in this result.
        let mut next_id = self
            .statements
            .iter()
            .map(|(id, _)| id.0)
            .chain(
                self.assumed
                    .iter()
                    .flat_map(|a| [a.id.0, a.drop_id.0, a.add_id.0]),
            )
            .chain(self.table_moves.iter().flat_map(|m| [m.id.0, m.drop_id.0]))
            .max()
            .map_or(0, |m| m + 1);
        let mut alloc_id = || {
            let id = ChangeId(next_id);
            next_id += 1;
            id
        };

        for decision in decisions {
            match decision {
                RenameDecision::Rename { from, to } => {
                    if let Some(ambiguous) = self
                        .unresolved
                        .iter()
                        .find(|a| a.removed == *from && a.candidates.iter().any(|c| c.added == *to))
                    {
                        let table_ref = ambiguous.table_ref.clone();
                        let id = alloc_id();
                        self.statements.push((
                            id,
                            db_backend.build(
                                TableAlterStatement::new()
                                    .table(table_ref)
                                    .rename_column(from.clone(), to.clone()),
                            ),
                        ));
                    }
                }
                RenameDecision::DropAndAdd { removed, .. } => {
                    if let Some(ambiguous) = self.unresolved.iter().find(|a| a.removed == *removed)
                    {
                        let table_ref = ambiguous.table_ref.clone();
                        let id = alloc_id();
                        self.statements.push((
                            id,
                            db_backend.build(
                                TableAlterStatement::new()
                                    .table(table_ref)
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
}

/// The schema-qualified table identity behind a [`sea_query::TableRef`], suitable
/// as a `HashMap`/`HashSet` key (unlike `TableRef` itself, which isn't `Eq`/`Hash`).
/// Discovery only ever produces the `TableRef::Table` variant.
fn table_key(table_ref: &sea_query::TableRef) -> sea_query::TableName {
    match table_ref {
        sea_query::TableRef::Table(name, _) => name.clone(),
        other => unreachable!("discovery only produces TableRef::Table, got {other:?}"),
    }
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
    let mut table_moves: Vec<AssumedTableMove> = Vec::new();

    // Ordered to satisfy FK / type constraints:
    // 1. CREATE SCHEMA — namespaces must exist before anything created inside them
    // 2. CREATE TYPE  — enum types must exist before tables that reference them
    // 3. CREATE TABLE / RENAME+MOVE TABLE — parents before children (ChangeSet records in sorted_tables order)
    // 4. ADD COLUMN
    // 5. ADD FK / ADD INDEX / ADD UNIQUE
    // 6. DROP FK / DROP UNIQUE
    // 7. DROP COLUMN
    // 8. DROP TABLE   — children before parents (ChangeSet records via sorted_table_drops)
    // 9. DROP TYPE    — after tables that referenced the type are gone
    interpret_schema_creates(&change_set.schemas, &mut statements);
    interpret_enum_creates(&change_set.enums, &mut statements);
    let moved_ids = interpret_table_moves(
        &change_set.tables,
        config,
        &mut statements,
        &mut suggestions,
        &mut table_moves,
    );
    interpret_table_creates(&change_set.tables, &moved_ids, &mut statements);
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
    interpret_table_drops(&change_set.tables, &moved_ids, config, &mut statements);
    interpret_enum_drops(&change_set.enums, config, &mut statements, &mut suggestions);

    InterpretResult {
        statements,
        warnings,
        suggestions,
        unresolved,
        assumed,
        table_moves,
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
/// Skips tables already handled by [`interpret_table_moves`].
fn interpret_table_creates(
    tables: &[TableChange],
    moved_ids: &HashSet<ChangeId>,
    statements: &mut Vec<(ChangeId, Statement)>,
) {
    for tc in tables {
        if moved_ids.contains(&tc.id) {
            continue;
        }
        if let TableChangeKind::Create { stmt, .. } = &tc.kind {
            statements.push((tc.id, stmt.clone()));
        }
    }
}

/// Column type equivalence for table-move signature matching. Treats
/// `Integer`/`BigInteger` as interchangeable: SQLite's `INTEGER PRIMARY KEY`
/// round-trips from discovery as `BigInteger`, even though an entity with an
/// `i32` primary key declares it as `Integer` — the create side of a move is
/// always the entity's own declared schema, while the drop side is always
/// DB-introspected, so this mismatch would otherwise silently defeat exact
/// signature matching for essentially every table with an auto-increment key.
fn column_types_equivalent(
    a: &Option<sea_query::ColumnType>,
    b: &Option<sea_query::ColumnType>,
) -> bool {
    use sea_query::ColumnType::{BigInteger, Integer};
    match (a, b) {
        (Some(Integer), Some(BigInteger)) | (Some(BigInteger), Some(Integer)) => true,
        _ => a == b,
    }
}

/// Column-name-and-type signature equality used for table-move detection,
/// tolerant of the entity-vs-introspected type quirks handled by
/// [`column_types_equivalent`].
fn signatures_match(
    a: &[(String, Option<sea_query::ColumnType>)],
    b: &[(String, Option<sea_query::ColumnType>)],
) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b.iter())
            .all(|((a_name, a_ty), (b_name, b_ty))| {
                a_name == b_name && column_types_equivalent(a_ty, b_ty)
            })
}

/// Detect a dropped table and a created table as the same table renamed
/// and/or moved to a different schema, by comparing column signatures.
/// Only ever matches a unique 1:1 pair (same rule as column rename detection).
///
/// Table renames are supported on every backend (`ALTER TABLE ... RENAME TO`
/// / MySQL's `RENAME TABLE`); schema-moves only make sense on Postgres, since
/// entities never carry a `schema_name` on MySQL/SQLite.
///
/// Returns the set of [`ChangeId`]s (of the original Create/Drop changes)
/// that were consumed by a detected move, so [`interpret_table_creates`] and
/// [`interpret_table_drops`] can skip them.
fn interpret_table_moves(
    tables: &[TableChange],
    config: &InterpretConfig,
    statements: &mut Vec<(ChangeId, Statement)>,
    suggestions: &mut Vec<DiscoverSuggestion>,
    table_moves: &mut Vec<AssumedTableMove>,
) -> HashSet<ChangeId> {
    let mut handled = HashSet::new();

    let creates: Vec<(
        ChangeId,
        &sea_query::TableRef,
        &Statement,
        &[(String, Option<sea_query::ColumnType>)],
    )> = tables
        .iter()
        .filter_map(|tc| match &tc.kind {
            TableChangeKind::Create {
                table_ref,
                stmt,
                columns,
            } => Some((tc.id, table_ref, stmt, columns.as_slice())),
            _ => None,
        })
        .collect();
    let drops: Vec<(
        ChangeId,
        &sea_query::TableName,
        &[(String, Option<sea_query::ColumnType>)],
    )> = tables
        .iter()
        .filter_map(|tc| match &tc.kind {
            TableChangeKind::Drop { table, columns } => Some((tc.id, table, columns.as_slice())),
            _ => None,
        })
        .collect();

    for &(drop_id, from_table, drop_columns) in &drops {
        // Match by exact column signature, uniquely on both sides — same
        // conservative rule as column rename detection, just without a
        // proximity heuristic (tables have no natural position to compare).
        let matches: Vec<_> = creates
            .iter()
            .filter(|(_, _, _, cols)| signatures_match(cols, drop_columns))
            .collect();
        if matches.len() != 1 {
            continue;
        }
        let &(create_id, to_table_ref, create_stmt, _) = matches[0];
        let reverse_matches = drops
            .iter()
            .filter(|(_, _, cols)| signatures_match(cols, drop_columns))
            .count();
        if reverse_matches != 1 {
            continue;
        }

        let to_table = table_key(to_table_ref);
        if from_table == &to_table {
            continue; // identical identity would already have matched in Phase 1
        }

        let name_changed = from_table.1 != to_table.1;
        let schema_changed = from_table.0 != to_table.0;
        // schema_changed can only be true on Postgres: schema_name is ignored
        // for MySQL/SQLite entities, so their table refs never carry one.
        if schema_changed && config.db_backend != DbBackend::Postgres {
            continue;
        }

        let from_ref = sea_query::TableRef::Table(from_table.clone(), None);
        let mut move_stmts: Vec<Statement> = Vec::new();
        let mut renamed_ref = from_ref.clone();
        if name_changed {
            let mut rename_stmt = sea_query::Table::rename();
            rename_stmt.table(
                from_ref.clone(),
                sea_query::Alias::new(to_table.1.to_string()),
            );
            move_stmts.push(config.db_backend.build(&rename_stmt));
            renamed_ref = sea_query::TableRef::Table(
                sea_query::TableName(from_table.0.clone(), to_table.1.clone()),
                None,
            );
        }
        if schema_changed {
            move_stmts.push(postgres_set_schema_stmt(&renamed_ref, &to_table.0));
        }
        if move_stmts.is_empty() {
            continue;
        }

        let drop_stmt = config.db_backend.build(
            sea_query::Table::drop()
                .table(sea_query::TableRef::Table(from_table.clone(), None))
                .if_exists(),
        );

        if config.assumptions {
            for (i, stmt) in move_stmts.into_iter().enumerate() {
                let id = if i == 0 { create_id } else { drop_id };
                statements.push((id, stmt));
            }
            table_moves.push(AssumedTableMove {
                from: from_ref,
                to: to_table_ref.clone(),
                id: create_id,
                drop_id,
                fallback_create: create_stmt.clone(),
                fallback_drop: drop_stmt,
            });
            handled.insert(create_id);
            handled.insert(drop_id);
        } else {
            suggestions.push(DiscoverSuggestion {
                kind: SuggestionKind::PossibleRename,
                message: format!(
                    "Table '{}' may have been renamed/moved to '{}' (identical columns). \
                     Enable assumptions to auto-apply.",
                    resolver::qualified_table_name(&from_ref),
                    resolver::qualified_table_name(to_table_ref),
                ),
                related_changes: vec![create_id, drop_id],
            });
        }
    }

    handled
}

/// Quote a Postgres identifier, doubling embedded quotes.
fn quote_pg_ident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

/// Build `ALTER TABLE ... SET SCHEMA ...` — Postgres-only, no [`sea_query`]
/// statement builder covers it.
fn postgres_set_schema_stmt(
    table_ref: &sea_query::TableRef,
    new_schema: &Option<sea_query::SchemaName>,
) -> Statement {
    let table_sql = match table_ref {
        sea_query::TableRef::Table(sea_query::TableName(Some(schema), table), _) => {
            format!(
                "{}.{}",
                quote_pg_ident(&schema.1.to_string()),
                quote_pg_ident(&table.to_string())
            )
        }
        sea_query::TableRef::Table(sea_query::TableName(None, table), _) => {
            quote_pg_ident(&table.to_string())
        }
        other => quote_pg_ident(&other.sea_orm_table().to_string()),
    };
    let schema_sql = match new_schema {
        Some(schema) => quote_pg_ident(&schema.1.to_string()),
        None => "public".to_string(),
    };
    Statement::from_string(
        DbBackend::Postgres,
        format!("ALTER TABLE {table_sql} SET SCHEMA {schema_sql}"),
    )
}

/// Emit DROP TABLE statements (children before parents via ChangeSet recording order).
/// Skips tables already handled by [`interpret_table_moves`].
fn interpret_table_drops(
    tables: &[TableChange],
    moved_ids: &HashSet<ChangeId>,
    config: &InterpretConfig,
    statements: &mut Vec<(ChangeId, Statement)>,
) {
    for tc in tables {
        if moved_ids.contains(&tc.id) {
            continue;
        }
        if let TableChangeKind::Drop { table, .. } = &tc.kind {
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
    let mut table_added: HashMap<sea_query::TableName, Vec<(ChangeId, AddedColumn, Statement)>> =
        Default::default();
    let mut table_removed: HashMap<
        sea_query::TableName,
        Vec<(ChangeId, RemovedColumn, Statement)>,
    > = Default::default();
    let mut table_refs: HashMap<sea_query::TableName, sea_query::TableRef> = Default::default();

    for cc in columns {
        let key = table_key(&cc.table_ref);
        table_refs
            .entry(key.clone())
            .or_insert_with(|| cc.table_ref.clone());
        let table_name = resolver::qualified_table_name(&cc.table_ref);
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
                            "Column '{table_name}.{column}' is NOT NULL with no default value. \
                             Existing rows will need data populated before or during this migration.",
                        ),
                        related_changes: vec![cc.id],
                    });
                }
                table_added.entry(key).or_default().push((
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
                table_removed.entry(key).or_default().push((
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
                            "Column '{table_name}.{from}' has a `renamed_from` annotation to '{to}'. \
                             Enable assumptions to auto-apply.",
                        ),
                        related_changes: vec![cc.id],
                    });
                }
            }
            ColumnChangeKind::CheckConstraintPresent { column } => {
                warnings.push(DiscoverWarning {
                    kind: WarningKind::CheckConstraintDiff,
                    message: format!(
                        "Column '{table_name}.{column}' has a CHECK constraint in entity definition. \
                         CHECK constraints cannot be automatically diffed — verify manually.",
                    ),
                    related_changes: vec![cc.id],
                });
            }
            ColumnChangeKind::TypeMismatch {
                column,
                existing_type,
                new_type,
            } => {
                warnings.push(DiscoverWarning {
                    kind: WarningKind::ColumnTypeMismatch,
                    message: format!(
                        "Column '{table_name}.{column}' has type {existing_type:?} in the \
                         database but is declared as {new_type:?} in the entity. This is left \
                         untouched — apply the change via a migration.",
                    ),
                    related_changes: vec![cc.id],
                });
            }
        }
    }

    // Rename detection per table
    let all_tables: HashSet<sea_query::TableName> = table_added
        .keys()
        .chain(table_removed.keys())
        .cloned()
        .collect();

    for table in &all_tables {
        let added = table_added.remove(table).unwrap_or_default();
        let removed = table_removed.remove(table).unwrap_or_default();
        let table_ref = table_refs[table].clone();
        let table_name = resolver::qualified_table_name(&table_ref);

        if (added.is_empty() && removed.is_empty()) {
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

        let resolution =
            resolver::resolve_renames(table_ref.clone(), resolver_added, resolver_removed);

        // Assumed renames
        for rename in &resolution.assumed {
            let add_id = added_ids[&rename.added];
            let drop_id = removed_ids[&rename.removed];

            if config.assumptions {
                add_stmts.push((
                    add_id,
                    config.db_backend.build(
                        TableAlterStatement::new()
                            .table(table_ref.clone())
                            .rename_column(rename.removed.clone(), rename.added.clone()),
                    ),
                ));
                suggestions.push(DiscoverSuggestion {
                    kind: SuggestionKind::PossibleRename,
                    message: format!(
                        "Column '{table_name}.{}' was auto-renamed to '{}' \
                         (same type, position proximity {}). Use `--rename` to override.",
                        rename.removed, rename.added, rename.proximity,
                    ),
                    related_changes: vec![add_id, drop_id],
                });
                assumed.push(AssumedRename {
                    table_ref: table_ref.clone(),
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
                        "Column '{table_name}.{}' may have been renamed to '{}' \
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

/// Emit variant-change / rename suggestions, and DROP TYPE for orphaned enums.
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
            EnumChangeKind::Rename {
                existing_name,
                new_name,
            } => {
                suggestions.push(DiscoverSuggestion {
                    kind: SuggestionKind::EnumRename,
                    message: format!(
                        "Enum type '{existing_name}' appears to have been renamed to '{new_name}'. \
                                             This requires `ALTER TYPE ... RENAME TO`.",
                    ),
                    related_changes: vec![ec.id],
                });
            }
            EnumChangeKind::Drop { stmt, .. } => {
                statements.push((ec.id, stmt.clone()));
            }
            EnumChangeKind::Create { .. } => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_query::IntoTableRef;

    fn qualified_table_ref() -> sea_query::TableRef {
        ("test", "person").into_table_ref()
    }

    fn other_qualified_table_ref() -> sea_query::TableRef {
        ("test1", "person").into_table_ref()
    }

    fn add_stmt(table_ref: sea_query::TableRef) -> Statement {
        DbBackend::Postgres.build(
            TableAlterStatement::new()
                .table(table_ref)
                .add_column(sea_query::ColumnDef::new(sea_query::Alias::new("name")).string()),
        )
    }

    fn drop_stmt(table_ref: sea_query::TableRef, column: &str) -> Statement {
        DbBackend::Postgres.build(
            TableAlterStatement::new()
                .table(table_ref)
                .drop_column(sea_query::Alias::new(column)),
        )
    }

    /// Regression test: each manually-resolved rename decision must get its
    /// own `ChangeId`, not a shared `usize::MAX` sentinel — otherwise a caller
    /// can't address one of them individually (e.g. via `exclude`).
    #[test]
    fn apply_rename_decisions_assigns_distinct_ids() {
        let mut result = InterpretResult {
            unresolved: vec![
                resolver::AmbiguousRename {
                    table_ref: qualified_table_ref(),
                    removed: "first_name".to_string(),
                    candidates: vec![resolver::RenameCandidate {
                        removed: "first_name".to_string(),
                        added: "full_name".to_string(),
                        proximity: 0,
                    }],
                },
                resolver::AmbiguousRename {
                    table_ref: qualified_table_ref(),
                    removed: "last_name".to_string(),
                    candidates: vec![resolver::RenameCandidate {
                        removed: "last_name".to_string(),
                        added: "surname".to_string(),
                        proximity: 0,
                    }],
                },
            ],
            ..Default::default()
        };

        result.apply_rename_decisions(
            &[
                RenameDecision::Rename {
                    from: "first_name".to_string(),
                    to: "full_name".to_string(),
                },
                RenameDecision::Rename {
                    from: "last_name".to_string(),
                    to: "surname".to_string(),
                },
            ],
            DbBackend::Postgres,
        );

        assert_eq!(result.statements.len(), 2);
        let ids: HashSet<_> = result.statements.iter().map(|(id, _)| *id).collect();
        assert_eq!(
            ids.len(),
            2,
            "each applied decision must get a distinct ChangeId: {:?}",
            result
                .statements
                .iter()
                .map(|(id, _)| id)
                .collect::<Vec<_>>()
        );
    }

    /// Regression test: an auto-assumed rename must keep the table's schema
    /// qualifier, not just the bare table name.
    #[test]
    fn assumed_rename_keeps_schema_qualifier() {
        let mut change_set = ChangeSet::default();
        change_set.record_column(
            qualified_table_ref(),
            ColumnChangeKind::Add {
                column: "name".to_string(),
                index: 0,
                column_type: Some(sea_query::ColumnType::String(sea_query::StringLen::None)),
                is_not_null: false,
                has_default: false,
                stmt: add_stmt(qualified_table_ref()),
            },
        );
        change_set.record_column(
            qualified_table_ref(),
            ColumnChangeKind::Drop {
                column: "first_name".to_string(),
                index: 0,
                column_type: Some(sea_query::ColumnType::String(sea_query::StringLen::None)),
                stmt: drop_stmt(qualified_table_ref(), "first_name"),
            },
        );

        let result = interpret(
            change_set,
            &InterpretConfig {
                db_backend: DbBackend::Postgres,
                assumptions: true,
            },
        );

        assert_eq!(result.assumed.len(), 1);
        assert_eq!(result.assumed[0].table_name(), "test.person");
        let sql = result.statements[0].1.sql.as_str();
        assert!(
            sql.contains(r#""test"."person""#),
            "expected schema-qualified table in RENAME COLUMN, got: {sql}"
        );

        // Rejecting the assumption must produce a schema-qualified DROP + ADD too.
        let id = result.assumed[0].id;
        let mut result = result;
        result.reject_assumed(id);
        for (_, stmt) in &result.statements {
            assert!(
                stmt.sql.contains(r#""test"."person""#),
                "expected schema-qualified table after rejecting assumption, got: {}",
                stmt.sql
            );
        }
    }

    /// Regression test: two tables sharing a bare name but living in
    /// different schemas must never be grouped together for rename detection
    /// — a drop in one schema must not be treated as a rename into an add in
    /// the other.
    #[test]
    fn same_table_name_different_schemas_not_cross_matched() {
        let mut change_set = ChangeSet::default();
        change_set.record_column(
            qualified_table_ref(),
            ColumnChangeKind::Drop {
                column: "first_name".to_string(),
                index: 0,
                column_type: Some(sea_query::ColumnType::String(sea_query::StringLen::None)),
                stmt: drop_stmt(qualified_table_ref(), "first_name"),
            },
        );
        change_set.record_column(
            other_qualified_table_ref(),
            ColumnChangeKind::Add {
                column: "name".to_string(),
                index: 0,
                column_type: Some(sea_query::ColumnType::String(sea_query::StringLen::None)),
                is_not_null: false,
                has_default: false,
                stmt: add_stmt(other_qualified_table_ref()),
            },
        );

        let result = interpret(
            change_set,
            &InterpretConfig {
                db_backend: DbBackend::Postgres,
                assumptions: true,
            },
        );

        assert!(
            result.assumed.is_empty(),
            "drop in one schema must not be assumed-renamed into an add in another, got: {:?}",
            result.assumed
        );
        let sql_all: String = result
            .statements
            .iter()
            .map(|(_, s)| s.sql.to_uppercase())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(!sql_all.contains("RENAME COLUMN"), "got: {sql_all}");
    }

    /// Regression test: resolving an ambiguous rename via `apply_rename_decisions`
    /// must also keep the schema qualifier.
    #[test]
    fn ambiguous_rename_decision_keeps_schema_qualifier() {
        let mut change_set = ChangeSet::default();
        // Two added candidates with the same type/proximity to force ambiguity.
        change_set.record_column(
            qualified_table_ref(),
            ColumnChangeKind::Add {
                column: "title".to_string(),
                index: 0,
                column_type: Some(sea_query::ColumnType::String(sea_query::StringLen::None)),
                is_not_null: false,
                has_default: false,
                stmt: add_stmt(qualified_table_ref()),
            },
        );
        change_set.record_column(
            qualified_table_ref(),
            ColumnChangeKind::Add {
                column: "label".to_string(),
                index: 1,
                column_type: Some(sea_query::ColumnType::String(sea_query::StringLen::None)),
                is_not_null: false,
                has_default: false,
                stmt: add_stmt(qualified_table_ref()),
            },
        );
        change_set.record_column(
            qualified_table_ref(),
            ColumnChangeKind::Drop {
                column: "name".to_string(),
                index: 0,
                column_type: Some(sea_query::ColumnType::String(sea_query::StringLen::None)),
                stmt: drop_stmt(qualified_table_ref(), "name"),
            },
        );

        let mut result = interpret(
            change_set,
            &InterpretConfig {
                db_backend: DbBackend::Postgres,
                assumptions: true,
            },
        );

        assert_eq!(result.unresolved.len(), 1);
        assert_eq!(result.unresolved[0].table_name(), "test.person");
        result.apply_rename_decisions(
            &[RenameDecision::Rename {
                from: "name".to_string(),
                to: "title".to_string(),
            }],
            DbBackend::Postgres,
        );

        let rename_stmt = result
            .statements
            .iter()
            .find(|(_, s)| s.sql.to_uppercase().contains("RENAME COLUMN"))
            .expect("rename statement should be present");
        assert!(
            rename_stmt.1.sql.contains(r#""test"."person""#),
            "expected schema-qualified table in resolved RENAME COLUMN, got: {}",
            rename_stmt.1.sql
        );
    }

    fn person_columns() -> Vec<(String, Option<sea_query::ColumnType>)> {
        vec![
            (
                "name".to_string(),
                Some(sea_query::ColumnType::String(sea_query::StringLen::None)),
            ),
            ("gender".to_string(), Some(sea_query::ColumnType::Text)),
        ]
    }

    fn record_table_create(
        change_set: &mut ChangeSet,
        table_ref: sea_query::TableRef,
        columns: Vec<(String, Option<sea_query::ColumnType>)>,
    ) -> ChangeId {
        let mut create_stmt = sea_query::Table::create();
        create_stmt
            .table(table_ref.clone())
            .col(sea_query::ColumnDef::new(sea_query::Alias::new("name")).string());
        let stmt = DbBackend::Postgres.build(&create_stmt);
        change_set.record_table(TableChangeKind::Create {
            table_ref,
            stmt,
            columns,
        })
    }

    fn record_table_drop(
        change_set: &mut ChangeSet,
        table: sea_query::TableName,
        columns: Vec<(String, Option<sea_query::ColumnType>)>,
    ) -> ChangeId {
        change_set.record_table(TableChangeKind::Drop { table, columns })
    }

    /// Regression test: a table renamed within the same (default) schema —
    /// same columns, different name — must be detected as a RENAME, not a
    /// CREATE + DROP.
    #[test]
    fn table_rename_same_schema_detected() {
        let mut change_set = ChangeSet::default();
        record_table_create(
            &mut change_set,
            sea_query::Alias::new("post1").into_table_ref(),
            person_columns(),
        );
        record_table_drop(
            &mut change_set,
            table_key(&sea_query::Alias::new("post").into_table_ref()),
            person_columns(),
        );

        let result = interpret(
            change_set,
            &InterpretConfig {
                db_backend: DbBackend::Postgres,
                assumptions: true,
            },
        );

        assert_eq!(result.table_moves.len(), 1);
        assert_eq!(result.table_moves[0].from_name(), "post");
        assert_eq!(result.table_moves[0].to_name(), "post1");
        assert_eq!(result.statements.len(), 1);
        let sql = result.statements[0].1.sql.to_uppercase();
        assert!(sql.contains("RENAME"), "got: {sql}");
        assert!(!sql.contains("CREATE TABLE"), "got: {sql}");
        assert!(!sql.contains("DROP TABLE"), "got: {sql}");
    }

    /// Regression test: a table moved to a different schema with the same
    /// name must be detected as a schema move (`SET SCHEMA`), not a
    /// CREATE + DROP.
    #[test]
    fn table_schema_move_detected() {
        let mut change_set = ChangeSet::default();
        record_table_create(&mut change_set, qualified_table_ref(), person_columns());
        record_table_drop(
            &mut change_set,
            table_key(&other_qualified_table_ref()),
            person_columns(),
        );

        let result = interpret(
            change_set,
            &InterpretConfig {
                db_backend: DbBackend::Postgres,
                assumptions: true,
            },
        );

        assert_eq!(result.table_moves.len(), 1);
        assert_eq!(result.table_moves[0].from_name(), "test1.person");
        assert_eq!(result.table_moves[0].to_name(), "test.person");
        assert_eq!(result.statements.len(), 1);
        let sql = &result.statements[0].1.sql;
        assert!(sql.to_uppercase().contains("SET SCHEMA"), "got: {sql}");
        assert!(sql.contains(r#""test1"."person""#), "got: {sql}");
        assert!(sql.contains(r#""test""#), "got: {sql}");

        // Rejecting must restore the original CREATE + DROP.
        let id = result.table_moves[0].id;
        let mut result = result;
        result.reject_table_move(id);
        let sql_all: String = result
            .statements
            .iter()
            .map(|(_, s)| s.sql.to_uppercase())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(sql_all.contains("CREATE TABLE"), "got: {sql_all}");
        assert!(sql_all.contains("DROP TABLE"), "got: {sql_all}");
    }

    /// A rename and a schema move at once (Postgres) must emit both statements.
    #[test]
    fn table_rename_and_move_detected() {
        let mut change_set = ChangeSet::default();
        record_table_create(
            &mut change_set,
            ("test", "person2").into_table_ref(),
            person_columns(),
        );
        record_table_drop(
            &mut change_set,
            table_key(&other_qualified_table_ref()),
            person_columns(),
        );

        let result = interpret(
            change_set,
            &InterpretConfig {
                db_backend: DbBackend::Postgres,
                assumptions: true,
            },
        );

        assert_eq!(result.table_moves.len(), 1);
        assert_eq!(result.statements.len(), 2);
        let sql_all: String = result
            .statements
            .iter()
            .map(|(_, s)| s.sql.to_uppercase())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(sql_all.contains("RENAME"), "got: {sql_all}");
        assert!(sql_all.contains("SET SCHEMA"), "got: {sql_all}");
    }

    /// A table move must not be auto-applied when `assumptions` is disabled —
    /// it should fall back to plain CREATE + DROP with a suggestion.
    #[test]
    fn table_move_not_applied_without_assumptions() {
        let mut change_set = ChangeSet::default();
        record_table_create(
            &mut change_set,
            sea_query::Alias::new("post1").into_table_ref(),
            person_columns(),
        );
        record_table_drop(
            &mut change_set,
            table_key(&sea_query::Alias::new("post").into_table_ref()),
            person_columns(),
        );

        let result = interpret(
            change_set,
            &InterpretConfig {
                db_backend: DbBackend::Postgres,
                assumptions: false,
            },
        );

        assert!(result.table_moves.is_empty());
        let sql_all: String = result
            .statements
            .iter()
            .map(|(_, s)| s.sql.to_uppercase())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(sql_all.contains("CREATE TABLE"), "got: {sql_all}");
        assert!(sql_all.contains("DROP TABLE"), "got: {sql_all}");
        assert!(
            result
                .suggestions
                .iter()
                .any(|s| s.kind == SuggestionKind::PossibleRename),
            "expected a PossibleRename suggestion"
        );
    }

    /// Tables with different column signatures must never be matched as a move.
    #[test]
    fn table_move_not_detected_for_different_columns() {
        let mut change_set = ChangeSet::default();
        record_table_create(
            &mut change_set,
            sea_query::Alias::new("post1").into_table_ref(),
            person_columns(),
        );
        record_table_drop(
            &mut change_set,
            table_key(&sea_query::Alias::new("post").into_table_ref()),
            vec![("id".to_string(), Some(sea_query::ColumnType::Integer))],
        );

        let result = interpret(
            change_set,
            &InterpretConfig {
                db_backend: DbBackend::Postgres,
                assumptions: true,
            },
        );

        assert!(result.table_moves.is_empty());
        let sql_all: String = result
            .statements
            .iter()
            .map(|(_, s)| s.sql.to_uppercase())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(sql_all.contains("CREATE TABLE"), "got: {sql_all}");
        assert!(sql_all.contains("DROP TABLE"), "got: {sql_all}");
    }

    /// MySQL doesn't have schema-qualified table refs in this codebase, but
    /// plain renames must still be detected.
    #[test]
    fn table_rename_detected_on_mysql() {
        let mut change_set = ChangeSet::default();
        record_table_create(
            &mut change_set,
            sea_query::Alias::new("post1").into_table_ref(),
            person_columns(),
        );
        record_table_drop(
            &mut change_set,
            table_key(&sea_query::Alias::new("post").into_table_ref()),
            person_columns(),
        );

        let result = interpret(
            change_set,
            &InterpretConfig {
                db_backend: DbBackend::MySql,
                assumptions: true,
            },
        );

        assert_eq!(result.table_moves.len(), 1);
        assert_eq!(result.statements.len(), 1);
        assert!(
            result.statements[0].1.sql.to_uppercase().contains("RENAME"),
            "got: {}",
            result.statements[0].1.sql
        );
    }
}
