use crate::Statement;
use sea_query::{ColumnType, TableName, TableRef};

/// Unique identifier for a recorded schema change.
#[cfg_attr(docsrs, doc(cfg(feature = "schema-sync")))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChangeId(pub usize);

// ── Table-level changes ──────────────────────────────────────────────────

/// A table-level change detected during schema discovery.
#[cfg_attr(docsrs, doc(cfg(feature = "schema-sync")))]
#[derive(Debug, Clone)]
pub struct TableChange {
    pub id: ChangeId,
    pub kind: TableChangeKind,
}

#[cfg_attr(docsrs, doc(cfg(feature = "schema-sync")))]
#[derive(Debug, Clone)]
pub enum TableChangeKind {
    /// Table exists in entities but not in the database.
    /// Carries the pre-built CREATE TABLE statement.
    Create { table: String, stmt: Statement },
    /// Table exists in the database but not in entities.
    Drop { table: TableName },
}

// ── Column-level changes ─────────────────────────────────────────────────

/// A column-level change detected during schema discovery.
#[cfg_attr(docsrs, doc(cfg(feature = "schema-sync")))]
#[derive(Debug, Clone)]
pub struct ColumnChange {
    pub id: ChangeId,
    pub table: String,
    pub kind: ColumnChangeKind,
}

#[cfg_attr(docsrs, doc(cfg(feature = "schema-sync")))]
#[derive(Debug, Clone)]
pub enum ColumnChangeKind {
    /// Column exists in entity but not in the database.
    Add {
        column: String,
        /// Position index in entity's column list.
        index: usize,
        column_type: Option<ColumnType>,
        is_not_null: bool,
        has_default: bool,
        /// Pre-built ALTER TABLE ADD COLUMN statement.
        stmt: Statement,
    },
    /// Column exists in the database but not in the entity.
    Drop {
        column: String,
        /// Position index in DB table's column list.
        index: usize,
        column_type: Option<ColumnType>,
        /// Pre-built ALTER TABLE DROP COLUMN statement.
        stmt: Statement,
    },
    /// An explicit rename annotation was found on a column.
    /// Carries the pre-built ALTER TABLE RENAME COLUMN statement.
    ExplicitRename {
        from: String,
        to: String,
        stmt: Statement,
    },
    /// A column has a CHECK constraint that cannot be automatically diffed.
    CheckConstraintPresent { column: String },
}

// ── Constraint-level changes ─────────────────────────────────────────────

/// A constraint/index-level change detected during schema discovery.
#[cfg_attr(docsrs, doc(cfg(feature = "schema-sync")))]
#[derive(Debug, Clone)]
pub struct ConstraintChange {
    pub id: ChangeId,
    pub table: String,
    pub kind: ConstraintChangeKind,
}

#[cfg_attr(docsrs, doc(cfg(feature = "schema-sync")))]
#[derive(Debug, Clone)]
pub enum ConstraintChangeKind {
    /// Pre-built ADD FOREIGN KEY statement.
    AddForeignKey { stmt: Statement },
    /// Pre-built ALTER TABLE DROP FOREIGN KEY statement.
    DropForeignKey { name: String, stmt: Statement },
    /// Pre-built CREATE INDEX statement.
    AddIndex { stmt: Statement },
    /// Pre-built CREATE UNIQUE INDEX statement.
    AddUniqueConstraint { column: String, stmt: Statement },
    /// Pre-built DROP INDEX / DROP CONSTRAINT statement.
    DropUniqueConstraint { name: String, stmt: Statement },
}

// ── Enum-level changes ───────────────────────────────────────────────────

/// An enum-level change detected during schema discovery.
#[cfg_attr(docsrs, doc(cfg(feature = "schema-sync")))]
#[derive(Debug, Clone)]
pub struct EnumChange {
    pub id: ChangeId,
    pub kind: EnumChangeKind,
}

#[cfg_attr(docsrs, doc(cfg(feature = "schema-sync")))]
#[derive(Debug, Clone)]
pub enum EnumChangeKind {
    /// Enum type exists in entities but not in the database.
    /// Carries the pre-built CREATE TYPE statement.
    Create { stmt: Statement },
    /// Same enum name, different variants.
    VariantChange {
        name: String,
        existing_sql: String,
        new_sql: String,
    },
    /// Same variants, different name — enum was renamed.
    Rename {
        existing_name: String,
        new_name: String,
    },
    /// Enum type exists in the database but not in any registered entity.
    /// Carries the pre-built DROP TYPE statement.
    Drop { name: String, stmt: Statement },
}

// ── Grouped change set ───────────────────────────────────────────────────

/// All recorded changes from Phase 1, grouped by origin.
///
/// Each group contains changes classified by their source: tables, columns,
/// constraints, and enums. Changes that require SQL carry pre-built
/// [`Statement`]s so that Phase 2 interpretation does not need access to
/// the original entity definitions.
#[cfg_attr(docsrs, doc(cfg(feature = "schema-sync")))]
#[derive(Debug, Clone, Default)]
pub struct ChangeSet {
    /// Table-level changes: creates and drops of entire tables.
    pub tables: Vec<TableChange>,
    /// Column-level changes: adds, drops, explicit renames, and CHECK constraint flags.
    pub columns: Vec<ColumnChange>,
    /// Constraint/index-level changes: foreign keys, indexes, and unique constraints.
    pub constraints: Vec<ConstraintChange>,
    /// Enum type changes: creates, variant diffs, and renames.
    pub enums: Vec<EnumChange>,
    /// SQL strings of already-recorded enum CREATE statements, used for deduplication.
    created_enum_sqls: Vec<String>,
    /// Internal counter for generating unique [`ChangeId`]s.
    next_id: usize,
}

impl ChangeSet {
    fn next_id(&mut self) -> ChangeId {
        let id = ChangeId(self.next_id);
        self.next_id += 1;
        id
    }

    pub fn record_table(&mut self, kind: TableChangeKind) -> ChangeId {
        let id = self.next_id();
        self.tables.push(TableChange { id, kind });
        id
    }

    pub fn record_column(&mut self, table: String, kind: ColumnChangeKind) -> ChangeId {
        let id = self.next_id();
        self.columns.push(ColumnChange { id, table, kind });
        id
    }

    pub fn record_constraint(&mut self, table: String, kind: ConstraintChangeKind) -> ChangeId {
        let id = self.next_id();
        self.constraints.push(ConstraintChange { id, table, kind });
        id
    }

    /// Record a new enum CREATE, deduplicating by SQL string.
    /// Returns `Some(ChangeId)` if recorded, `None` if already seen.
    pub fn record_enum_create(&mut self, sql: &str, stmt: Statement) -> Option<ChangeId> {
        if self.created_enum_sqls.iter().any(|s| s == sql) {
            return None;
        }
        self.created_enum_sqls.push(sql.to_owned());
        Some(self.record_enum(EnumChangeKind::Create { stmt }))
    }

    pub fn record_enum(&mut self, kind: EnumChangeKind) -> ChangeId {
        let id = self.next_id();
        self.enums.push(EnumChange { id, kind });
        id
    }

    /// Collect all pre-built statements from every group, in recording order.
    /// Useful for simple apply-all scenarios like `sync()`.
    pub fn statements(self) -> Vec<Statement> {
        let mut stmts = Vec::new();

        for ec in self.enums {
            match ec.kind {
                EnumChangeKind::Create { stmt } => stmts.push(stmt),
                EnumChangeKind::Drop { .. }
                | EnumChangeKind::VariantChange { .. }
                | EnumChangeKind::Rename { .. } => {} // sync never drops or modifies enums
            }
        }
        for tc in self.tables {
            match tc.kind {
                TableChangeKind::Create { stmt, .. } => stmts.push(stmt),
                TableChangeKind::Drop { .. } => {} // sync never drops
            }
        }
        for cc in self.columns {
            match cc.kind {
                ColumnChangeKind::Add { stmt, .. }
                | ColumnChangeKind::ExplicitRename { stmt, .. } => stmts.push(stmt),
                ColumnChangeKind::Drop { .. } | ColumnChangeKind::CheckConstraintPresent { .. } => {
                }
            }
        }
        for cc in self.constraints {
            match cc.kind {
                ConstraintChangeKind::AddForeignKey { stmt }
                | ConstraintChangeKind::AddIndex { stmt }
                | ConstraintChangeKind::AddUniqueConstraint { stmt, .. } => stmts.push(stmt),
                ConstraintChangeKind::DropForeignKey { .. }
                | ConstraintChangeKind::DropUniqueConstraint { .. } => {}
            }
        }

        stmts
    }
}
