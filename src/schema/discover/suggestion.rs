//! Suggested fixes emitted during schema discovery.
//!
//! Suggestions are heuristic-powered proposals for renames and other changes
//! that the system can reasonably detect. They are only generated when the
//! `assumptions` flag is enabled. Suggestions reference the changes they
//! act upon via [`ChangeId`].

use super::changes::ChangeId;

/// A suggested fix detected by heuristic analysis during schema discovery.
#[cfg_attr(docsrs, doc(cfg(feature = "schema-sync")))]
#[derive(Debug, Clone)]
pub struct DiscoverSuggestion {
    /// The category of suggestion.
    pub kind: SuggestionKind,
    /// Human-readable description of the suggested change.
    pub message: String,
    /// IDs of the changes this suggestion relates to (e.g. the ADD + DROP that form a rename).
    pub related_changes: Vec<ChangeId>,
}

/// Categories of schema discovery suggestions.
///
/// Suggestions are only generated when `assumptions` is enabled.
/// They represent changes that the system can heuristically detect and auto-apply.
#[cfg_attr(docsrs, doc(cfg(feature = "schema-sync")))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuggestionKind {
    /// A column was removed and another with the same type was added — obvious rename (auto-assumed).
    PossibleRename,
    /// An enum type exists in both DB and entities but with different variants.
    EnumVariantChange,
    /// An enum type appears to have been renamed (same variants, different name).
    EnumRename,
}
