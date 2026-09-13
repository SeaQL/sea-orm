use super::changes::{ChangeSet, EnumChangeKind};
use super::resolver::{self, extract_enum_type_name};
use crate::DbBackend;
use sea_query::extension::postgres::TypeCreateStatement;

/// Phase 1: Record enum types in the database that have no matching entity.
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
        let Some(new_name) = extract_enum_type_name(new_sql) else {
            continue;
        };

        // Same-name match: enum type names are unique in the database, so this
        // is either identical or a variant change on that same enum — never
        // ambiguous with any other existing enum.
        let same_name = existing.iter().find(|e| {
            extract_enum_type_name(&db_backend.build(*e).sql).as_deref() == Some(new_name.as_str())
        });
        if let Some(existing_enum) = same_name {
            let existing_stmt = db_backend.build(existing_enum);
            if existing_stmt.sql != *new_sql {
                changes.record_enum(EnumChangeKind::VariantChange {
                    name: new_name,
                    existing_sql: existing_stmt.sql,
                    new_sql: new_sql.clone(),
                });
            }
            continue;
        }

        // No same-name match: only treat this as a rename if exactly one
        // existing enum shares its variant set — same conservative rule as
        // column rename detection. A non-unique match is left as a plain
        // create rather than guessing which existing enum it was renamed from.
        let new_variants = resolver::extract_enum_variants(new_sql);
        let rename_candidates: Vec<&TypeCreateStatement> = if new_variants.is_empty() {
            Vec::new()
        } else {
            existing
                .iter()
                .filter(|e| {
                    resolver::extract_enum_variants(&db_backend.build(*e).sql) == new_variants
                })
                .collect()
        };

        if let [existing_enum] = rename_candidates[..]
            && let Some(existing_name) =
                extract_enum_type_name(&db_backend.build(existing_enum).sql)
        {
            changes.record_enum(EnumChangeKind::Rename {
                existing_name,
                new_name,
            });
            continue;
        }

        let sql = new_sql.clone();
        changes.record_enum_create(&sql, new_stmt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enum_stmt(name: &str, variants: &[&str]) -> TypeCreateStatement {
        sea_query::extension::postgres::Type::create()
            .as_enum(sea_query::Alias::new(name))
            .values(variants.iter().map(|v| sea_query::Alias::new(*v)))
            .to_owned()
    }

    /// Two existing enums share `status`'s variant set — the rename must be
    /// left ambiguous (recorded as a plain create) rather than guessing.
    #[test]
    fn ambiguous_variant_match_is_not_treated_as_rename() {
        let backend = DbBackend::Postgres;
        let existing = vec![
            enum_stmt("priority_level", &["low", "high"]),
            enum_stmt("other_status", &["low", "high"]),
        ];
        let new_enum = enum_stmt("status", &["low", "high"]);

        let mut changes = ChangeSet::default();
        record_enum_changes(&[new_enum], backend, &existing, &mut changes);

        assert!(
            changes
                .enums
                .iter()
                .all(|e| !matches!(e.kind, EnumChangeKind::Rename { .. })),
            "must not guess a rename between two equally-plausible candidates: {:?}",
            changes.enums
        );
        assert!(
            changes
                .enums
                .iter()
                .any(|e| matches!(e.kind, EnumChangeKind::Create { .. })),
            "ambiguous match should fall back to create: {:?}",
            changes.enums
        );
    }

    /// Exactly one existing enum shares `status`'s variant set — this is the
    /// unambiguous rename case and must still be detected.
    #[test]
    fn unique_variant_match_is_treated_as_rename() {
        let backend = DbBackend::Postgres;
        let existing = vec![enum_stmt("priority_level", &["low", "high"])];
        let new_enum = enum_stmt("status", &["low", "high"]);

        let mut changes = ChangeSet::default();
        record_enum_changes(&[new_enum], backend, &existing, &mut changes);

        assert!(
            changes.enums.iter().any(|e| matches!(
                &e.kind,
                EnumChangeKind::Rename { existing_name, new_name }
                    if existing_name == "priority_level" && new_name == "status"
            )),
            "expected a rename from priority_level to status: {:?}",
            changes.enums
        );
    }
}
