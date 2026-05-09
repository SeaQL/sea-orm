//! Warnings emitted during schema discovery.
//!
//! Warnings are always-on alerts about changes that cannot be handled automatically
//! and require manual intervention — typically data migration concerns.
//! Warnings can reference specific changes by their [`ChangeId`].

use super::changes::ChangeId;

/// A warning emitted during schema discovery about a change requiring manual attention.
#[cfg_attr(docsrs, doc(cfg(feature = "schema-sync")))]
#[derive(Debug, Clone)]
pub struct DiscoverWarning {
    /// The category of warning.
    pub kind: WarningKind,
    /// Human-readable description of the concern.
    pub message: String,
    /// IDs of the changes this warning relates to, if any.
    pub related_changes: Vec<ChangeId>,
}

/// Categories of schema discovery warnings.
///
/// Warnings are always emitted regardless of the `assumptions` flag.
/// They represent situations that cannot be automatically resolved.
#[cfg_attr(docsrs, doc(cfg(feature = "schema-sync")))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WarningKind {
    /// A CHECK constraint exists in entity definition but cannot be automatically diffed.
    CheckConstraintDiff,
    /// A column is being added with NOT NULL and no default — existing rows need data populated.
    NotNullNoDefault,
}
