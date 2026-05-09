//! Heuristic rename detection and enum change resolution for schema discovery.
//!
//! This module contains pure functions that take raw added/removed column lists
//! and produce structured rename decisions. No I/O or user interaction happens here.

use sea_query::ColumnType;

/// A column that exists in the entity but not in the database.
#[derive(Debug, Clone)]
pub struct AddedColumn {
    /// Position index in the entity's column list.
    pub index: usize,
    /// Column name.
    pub name: String,
    /// Column type (if available from the entity definition).
    pub column_type: Option<ColumnType>,
}

/// A column that exists in the database but not in the entity.
#[derive(Debug, Clone)]
pub struct RemovedColumn {
    /// Position index in the database table's column list.
    pub index: usize,
    /// Column name.
    pub name: String,
    /// Column type (if available from schema discovery).
    pub column_type: Option<ColumnType>,
}

/// A single rename candidate pairing a removed column with an added column.
#[derive(Debug, Clone)]
pub struct RenameCandidate {
    /// The name of the removed (old) column.
    pub removed: String,
    /// The name of the added (new) column.
    pub added: String,
    /// Positional distance between the two columns.
    pub proximity: usize,
}

/// An ambiguous rename where multiple candidates exist for a removed column.
#[derive(Debug, Clone)]
pub struct AmbiguousRename {
    /// The table this rename occurs in.
    pub table: String,
    /// The name of the removed column.
    pub removed: String,
    /// All possible added columns it could be renamed to.
    pub candidates: Vec<RenameCandidate>,
}

/// The result of rename resolution.
#[derive(Debug, Clone, Default)]
pub struct RenameResolution {
    /// Obvious renames (1:1 mapping, same type, close proximity) — auto-decided.
    pub assumed: Vec<RenameCandidate>,
    /// Ambiguous renames (multiple candidates) — need user input.
    pub ambiguous: Vec<AmbiguousRename>,
    /// Genuinely new columns (no rename match).
    pub remaining_added: Vec<AddedColumn>,
    /// Genuinely removed columns (no rename match).
    pub remaining_removed: Vec<RemovedColumn>,
}

/// The kind of enum change detected between existing and new definitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumChange {
    /// Same enum name, different variants.
    VariantChange {
        /// The enum type name.
        name: String,
        /// The existing CREATE TYPE SQL.
        existing_sql: String,
        /// The new CREATE TYPE SQL.
        new_sql: String,
    },
    /// Different enum name with same variants (enum was renamed).
    NameChange {
        /// The existing enum type name.
        existing_name: String,
        /// The new enum type name.
        new_name: String,
    },
}

/// Check if two column types are compatible for rename detection.
/// Treats String variants (String, Text) as equivalent.
pub fn types_compatible(a: Option<&ColumnType>, b: Option<&ColumnType>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => {
            if a == b {
                return true;
            }
            // Treat all String/Text variants as compatible
            matches!(
                (a, b),
                (
                    ColumnType::String(_) | ColumnType::Text,
                    ColumnType::String(_) | ColumnType::Text,
                )
            )
        }
        _ => false,
    }
}

/// Resolve renames from lists of added and removed columns.
///
/// For each removed column, find added columns with compatible types and proximity ≤ 2.
/// - If exactly one match for both sides (1:1, neither claimed elsewhere) → assumed rename.
/// - If multiple candidates → ambiguous rename.
/// - Unmatched columns go to remaining_added / remaining_removed.
pub fn resolve_renames(
    table: &str,
    added: Vec<AddedColumn>,
    removed: Vec<RemovedColumn>,
) -> RenameResolution {
    let mut resolution = RenameResolution::default();

    // For each removed column, collect all compatible added candidates within proximity
    let mut removed_candidates: Vec<(usize, Vec<(usize, RenameCandidate)>)> = Vec::new();

    for (ri, rem) in removed.iter().enumerate() {
        let mut candidates = Vec::new();
        for (ai, add) in added.iter().enumerate() {
            let proximity = (rem.index as isize - add.index as isize).unsigned_abs();
            if proximity <= 2
                && types_compatible(rem.column_type.as_ref(), add.column_type.as_ref())
            {
                candidates.push((
                    ai,
                    RenameCandidate {
                        removed: rem.name.clone(),
                        added: add.name.clone(),
                        proximity,
                    },
                ));
            }
        }
        removed_candidates.push((ri, candidates));
    }

    // First pass: identify obvious 1:1 renames
    let mut claimed_added: Vec<usize> = Vec::new();
    let mut claimed_removed: Vec<usize> = Vec::new();

    // Sort by number of candidates (fewest first) to greedily resolve unambiguous ones
    let mut sorted_by_candidates: Vec<_> = removed_candidates.iter().collect();
    sorted_by_candidates.sort_by_key(|(_, cands)| cands.len());

    for (ri, candidates) in &sorted_by_candidates {
        if claimed_removed.contains(ri) {
            continue;
        }
        // Filter out already-claimed added columns
        let available: Vec<_> = candidates
            .iter()
            .filter(|(ai, _)| !claimed_added.contains(ai))
            .collect();

        if available.len() == 1 {
            // Check the reverse: is this added column also only matched by one removed column?
            let ai = available[0].0;
            let reverse_count = removed_candidates
                .iter()
                .filter(|(other_ri, other_cands)| {
                    !claimed_removed.contains(other_ri)
                        && other_cands.iter().any(|(other_ai, _)| {
                            *other_ai == ai && !claimed_added.contains(other_ai)
                        })
                })
                .count();

            if reverse_count == 1 {
                // Unique 1:1 mapping → assumed rename
                resolution.assumed.push(available[0].1.clone());
                claimed_added.push(ai);
                claimed_removed.push(*ri);
            }
        }
    }

    // Second pass: collect ambiguous renames from unclaimed removed columns with candidates
    for (ri, candidates) in &removed_candidates {
        if claimed_removed.contains(ri) {
            continue;
        }
        let available: Vec<_> = candidates
            .iter()
            .filter(|(ai, _)| !claimed_added.contains(ai))
            .map(|(_, c)| c.clone())
            .collect();

        if available.len() > 1 {
            resolution.ambiguous.push(AmbiguousRename {
                table: table.to_string(),
                removed: removed[*ri].name.clone(),
                candidates: available,
            });
            claimed_removed.push(*ri);
            // Don't claim the added columns — the user will decide
        }
    }

    // Remaining: unclaimed added and removed columns
    for (ai, add) in added.iter().enumerate() {
        if !claimed_added.contains(&ai) {
            // Check if this added column is referenced in an ambiguous rename
            let in_ambiguous = resolution
                .ambiguous
                .iter()
                .any(|a| a.candidates.iter().any(|c| c.added == add.name));
            if !in_ambiguous {
                resolution.remaining_added.push(add.clone());
            }
        }
    }

    for (ri, rem) in removed.iter().enumerate() {
        if !claimed_removed.contains(&ri) {
            resolution.remaining_removed.push(rem.clone());
        }
    }

    resolution
}

/// Detect enum changes between existing and new SQL definitions.
/// Compares two `CREATE TYPE ... AS ENUM (...)` SQL strings and returns
/// the kind of change detected, if any.
pub fn detect_enum_change(existing_sql: &str, new_sql: &str) -> Option<EnumChange> {
    let existing_name = extract_enum_type_name(existing_sql)?;
    let new_name = extract_enum_type_name(new_sql)?;

    if existing_name == new_name && existing_sql != new_sql {
        Some(EnumChange::VariantChange {
            name: existing_name,
            existing_sql: existing_sql.to_string(),
            new_sql: new_sql.to_string(),
        })
    } else if existing_name != new_name {
        // Extract variants to check if they match
        let existing_variants = extract_enum_variants(existing_sql);
        let new_variants = extract_enum_variants(new_sql);
        if existing_variants == new_variants && !existing_variants.is_empty() {
            Some(EnumChange::NameChange {
                existing_name,
                new_name,
            })
        } else {
            None
        }
    } else {
        None
    }
}

/// Extract the type name from a `CREATE TYPE "name" AS ENUM (...)` SQL string.
pub fn extract_enum_type_name(sql: &str) -> Option<String> {
    let upper = sql.to_uppercase();
    let crt_type_pos = upper.find("CREATE TYPE")?;
    let as_enum_pos = upper.find("AS ENUM")?;
    if as_enum_pos <= crt_type_pos {
        return None;
    }

    let between = sql[crt_type_pos + "CREATE TYPE".len()..as_enum_pos].trim();
    // Extract quoted or unquoted identifier
    let name = if let Some(stripped) = between.strip_prefix('"') {
        let end = stripped.find('"')?;
        &stripped[..end]
    } else {
        between.split_whitespace().next()?
    };
    Some(name.to_string())
}

/// Extract enum variant strings from a CREATE TYPE ... AS ENUM (...) SQL statement.
fn extract_enum_variants(sql: &str) -> Vec<String> {
    let upper = sql.to_uppercase();
    let Some(paren_start) = upper.find("AS ENUM") else {
        return Vec::new();
    };
    let rest = &sql[paren_start..];
    let Some(open) = rest.find('(') else {
        return Vec::new();
    };
    let Some(close) = rest.find(')') else {
        return Vec::new();
    };
    let inner = &rest[open + 1..close];
    inner
        .split(',')
        .map(|s| s.trim().trim_matches('\'').trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn added(index: usize, name: &str, col_type: Option<ColumnType>) -> AddedColumn {
        AddedColumn {
            index,
            name: name.to_string(),
            column_type: col_type,
        }
    }

    fn removed(index: usize, name: &str, col_type: Option<ColumnType>) -> RemovedColumn {
        RemovedColumn {
            index,
            name: name.to_string(),
            column_type: col_type,
        }
    }

    #[test]
    fn test_single_obvious_rename() {
        let added_cols = vec![added(
            1,
            "title",
            Some(ColumnType::String(sea_query::StringLen::None)),
        )];
        let removed_cols = vec![removed(
            1,
            "name",
            Some(ColumnType::String(sea_query::StringLen::None)),
        )];

        let result = resolve_renames("cake", added_cols, removed_cols);

        assert_eq!(result.assumed.len(), 1);
        assert_eq!(result.assumed[0].removed, "name");
        assert_eq!(result.assumed[0].added, "title");
        assert_eq!(result.assumed[0].proximity, 0);
        assert!(result.ambiguous.is_empty());
        assert!(result.remaining_added.is_empty());
        assert!(result.remaining_removed.is_empty());
    }

    #[test]
    fn test_no_rename_type_mismatch() {
        let added_cols = vec![added(1, "count", Some(ColumnType::Integer))];
        let removed_cols = vec![removed(
            1,
            "name",
            Some(ColumnType::String(sea_query::StringLen::None)),
        )];

        let result = resolve_renames("cake", added_cols, removed_cols);

        assert!(result.assumed.is_empty());
        assert!(result.ambiguous.is_empty());
        assert_eq!(result.remaining_added.len(), 1);
        assert_eq!(result.remaining_removed.len(), 1);
    }

    #[test]
    fn test_no_rename_too_far() {
        let added_cols = vec![added(
            5,
            "title",
            Some(ColumnType::String(sea_query::StringLen::None)),
        )];
        let removed_cols = vec![removed(
            1,
            "name",
            Some(ColumnType::String(sea_query::StringLen::None)),
        )];

        let result = resolve_renames("cake", added_cols, removed_cols);

        assert!(result.assumed.is_empty());
        assert!(result.ambiguous.is_empty());
        assert_eq!(result.remaining_added.len(), 1);
        assert_eq!(result.remaining_removed.len(), 1);
    }

    #[test]
    fn test_ambiguous_multiple_candidates() {
        // One removed column, two added columns with same type and close proximity
        let added_cols = vec![
            added(
                1,
                "title",
                Some(ColumnType::String(sea_query::StringLen::None)),
            ),
            added(
                2,
                "label",
                Some(ColumnType::String(sea_query::StringLen::None)),
            ),
        ];
        let removed_cols = vec![removed(
            1,
            "name",
            Some(ColumnType::String(sea_query::StringLen::None)),
        )];

        let result = resolve_renames("cake", added_cols, removed_cols);

        assert!(result.assumed.is_empty());
        assert_eq!(result.ambiguous.len(), 1);
        assert_eq!(result.ambiguous[0].table, "cake");
        assert_eq!(result.ambiguous[0].removed, "name");
        assert_eq!(result.ambiguous[0].candidates.len(), 2);
    }

    #[test]
    fn test_multiple_independent_renames() {
        // Two removed + two added, each pair is uniquely matched
        let added_cols = vec![
            added(
                1,
                "title",
                Some(ColumnType::String(sea_query::StringLen::None)),
            ),
            added(3, "weight_kg", Some(ColumnType::Integer)),
        ];
        let removed_cols = vec![
            removed(
                1,
                "name",
                Some(ColumnType::String(sea_query::StringLen::None)),
            ),
            removed(3, "weight", Some(ColumnType::Integer)),
        ];

        let result = resolve_renames("product", added_cols, removed_cols);

        assert_eq!(result.assumed.len(), 2);
        assert!(result.ambiguous.is_empty());
        assert!(result.remaining_added.is_empty());
        assert!(result.remaining_removed.is_empty());

        let names: Vec<_> = result.assumed.iter().map(|r| r.removed.as_str()).collect();
        assert!(names.contains(&"name"));
        assert!(names.contains(&"weight"));
    }

    #[test]
    fn test_string_text_type_compatibility() {
        let added_cols = vec![added(1, "description", Some(ColumnType::Text))];
        let removed_cols = vec![removed(
            1,
            "desc",
            Some(ColumnType::String(sea_query::StringLen::None)),
        )];

        let result = resolve_renames("item", added_cols, removed_cols);

        assert_eq!(result.assumed.len(), 1);
        assert_eq!(result.assumed[0].removed, "desc");
        assert_eq!(result.assumed[0].added, "description");
    }

    #[test]
    fn test_proximity_at_boundary() {
        // Proximity exactly 2 — should still match
        let added_cols = vec![added(
            3,
            "title",
            Some(ColumnType::String(sea_query::StringLen::None)),
        )];
        let removed_cols = vec![removed(
            1,
            "name",
            Some(ColumnType::String(sea_query::StringLen::None)),
        )];

        let result = resolve_renames("cake", added_cols, removed_cols);
        assert_eq!(result.assumed.len(), 1);
        assert_eq!(result.assumed[0].proximity, 2);

        // Proximity 3 — should NOT match
        let added_cols = vec![added(
            4,
            "title",
            Some(ColumnType::String(sea_query::StringLen::None)),
        )];
        let removed_cols = vec![removed(
            1,
            "name",
            Some(ColumnType::String(sea_query::StringLen::None)),
        )];

        let result = resolve_renames("cake", added_cols, removed_cols);
        assert!(result.assumed.is_empty());
    }

    #[test]
    fn test_no_columns_produces_empty_resolution() {
        let result = resolve_renames("empty", vec![], vec![]);
        assert!(result.assumed.is_empty());
        assert!(result.ambiguous.is_empty());
        assert!(result.remaining_added.is_empty());
        assert!(result.remaining_removed.is_empty());
    }

    #[test]
    fn test_only_added_columns() {
        let added_cols = vec![
            added(
                1,
                "new_col",
                Some(ColumnType::String(sea_query::StringLen::None)),
            ),
            added(2, "another", Some(ColumnType::Integer)),
        ];

        let result = resolve_renames("t", added_cols, vec![]);
        assert!(result.assumed.is_empty());
        assert!(result.ambiguous.is_empty());
        assert_eq!(result.remaining_added.len(), 2);
        assert!(result.remaining_removed.is_empty());
    }

    #[test]
    fn test_only_removed_columns() {
        let removed_cols = vec![removed(
            1,
            "old_col",
            Some(ColumnType::String(sea_query::StringLen::None)),
        )];

        let result = resolve_renames("t", vec![], removed_cols);
        assert!(result.assumed.is_empty());
        assert!(result.ambiguous.is_empty());
        assert!(result.remaining_added.is_empty());
        assert_eq!(result.remaining_removed.len(), 1);
    }

    #[test]
    fn test_enum_variant_change() {
        let existing = r#"CREATE TYPE "mood" AS ENUM ('happy', 'sad')"#;
        let new = r#"CREATE TYPE "mood" AS ENUM ('happy', 'sad', 'neutral')"#;

        let change = detect_enum_change(existing, new);
        assert!(change.is_some());
        match change.unwrap() {
            EnumChange::VariantChange { name, .. } => {
                assert_eq!(name, "mood");
            }
            _ => panic!("expected VariantChange"),
        }
    }

    #[test]
    fn test_enum_rename() {
        let existing = r#"CREATE TYPE "mood" AS ENUM ('happy', 'sad')"#;
        let new = r#"CREATE TYPE "feeling" AS ENUM ('happy', 'sad')"#;

        let change = detect_enum_change(existing, new);
        assert!(change.is_some());
        match change.unwrap() {
            EnumChange::NameChange {
                existing_name,
                new_name,
            } => {
                assert_eq!(existing_name, "mood");
                assert_eq!(new_name, "feeling");
            }
            _ => panic!("expected NameChange"),
        }
    }

    #[test]
    fn test_enum_no_change() {
        let sql = r#"CREATE TYPE "mood" AS ENUM ('happy', 'sad')"#;
        assert!(detect_enum_change(sql, sql).is_none());
    }

    #[test]
    fn test_enum_completely_different() {
        let existing = r#"CREATE TYPE "mood" AS ENUM ('happy', 'sad')"#;
        let new = r#"CREATE TYPE "color" AS ENUM ('red', 'blue')"#;

        // Different name AND different variants — no match
        assert!(detect_enum_change(existing, new).is_none());
    }

    #[test]
    fn test_types_compatible_same() {
        assert!(types_compatible(
            Some(&ColumnType::Integer),
            Some(&ColumnType::Integer)
        ));
    }

    #[test]
    fn test_types_compatible_string_text() {
        assert!(types_compatible(
            Some(&ColumnType::String(sea_query::StringLen::None)),
            Some(&ColumnType::Text)
        ));
        assert!(types_compatible(
            Some(&ColumnType::Text),
            Some(&ColumnType::String(sea_query::StringLen::N(255)))
        ));
    }

    #[test]
    fn test_types_compatible_none() {
        assert!(!types_compatible(None, Some(&ColumnType::Integer)));
        assert!(!types_compatible(Some(&ColumnType::Integer), None));
        assert!(!types_compatible(None, None));
    }

    #[test]
    fn test_types_incompatible() {
        assert!(!types_compatible(
            Some(&ColumnType::Integer),
            Some(&ColumnType::String(sea_query::StringLen::None))
        ));
    }
}
