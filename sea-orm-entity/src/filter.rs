use sea_orm::Statement;

/// Filter out DROP TABLE statements for `protected_table` from a list of discovered statements
pub fn filter_protected_drops(stmts: Vec<Statement>, protected_table: &str) -> Vec<Statement> {
    let protected_upper = protected_table.to_uppercase();
    stmts
        .into_iter()
        .filter(|stmt| {
            let upper = stmt.sql.to_uppercase();
            if upper.contains("DROP TABLE") {
                !is_drop_of(upper.as_str(), &protected_upper)
            } else {
                true
            }
        })
        .collect()
}

/// Returns true if `sql_upper` is a DROP TABLE statement targeting `table_upper`.
/// Handles all three quoting styles (double-quote, backtick, unquoted) and the
/// optional `IF EXISTS` clause so that no backend-specific variant slips through.
fn is_drop_of(sql_upper: &str, table_upper: &str) -> bool {
    sql_upper.contains(&format!("\"{}\"", table_upper))
        || sql_upper.contains(&format!("`{}`", table_upper))
        || sql_upper.contains(&format!(" {} ", table_upper))
        || sql_upper.ends_with(&format!(" {}", table_upper))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DbBackend, Statement};

    fn stmt(sql: &str) -> Statement {
        Statement::from_string(DbBackend::Sqlite, sql.to_owned())
    }

    #[test]
    fn test_removes_double_quoted_drop() {
        let stmts = vec![
            stmt(r#"DROP TABLE IF EXISTS "seaql_migrations""#),
            stmt(r#"DROP TABLE IF EXISTS "fruit""#),
        ];
        let filtered = filter_protected_drops(stmts, "seaql_migrations");
        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].sql.contains("fruit"));
    }

    #[test]
    fn test_removes_backtick_quoted_drop() {
        let stmts = vec![
            stmt("DROP TABLE IF EXISTS `seaql_migrations`"),
            stmt("DROP TABLE IF EXISTS `cake`"),
        ];
        let filtered = filter_protected_drops(stmts, "seaql_migrations");
        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].sql.contains("cake"));
    }

    #[test]
    fn test_removes_unquoted_drop() {
        let stmts = vec![
            stmt("DROP TABLE IF EXISTS seaql_migrations"),
            stmt("DROP TABLE IF EXISTS cake"),
        ];
        let filtered = filter_protected_drops(stmts, "seaql_migrations");
        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].sql.contains("cake"));
    }

    #[test]
    fn test_does_not_remove_partial_name_match() {
        let stmts = vec![stmt(r#"DROP TABLE IF EXISTS "seaql_migrations_old""#)];
        let filtered = filter_protected_drops(stmts, "seaql_migrations");
        assert_eq!(filtered.len(), 1, "partial name match must not be filtered");
    }

    #[test]
    fn test_non_drop_stmts_pass_through() {
        let stmts = vec![
            stmt(r#"CREATE TABLE "cake" ( "id" integer NOT NULL )"#),
            stmt(r#"ALTER TABLE "fruit" ADD COLUMN "weight" integer"#),
        ];
        let filtered = filter_protected_drops(stmts, "seaql_migrations");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_custom_migration_table_name() {
        let stmts = vec![
            stmt(r#"DROP TABLE IF EXISTS "my_migrations""#),
            stmt(r#"DROP TABLE IF EXISTS "cake""#),
        ];
        let filtered = filter_protected_drops(stmts, "my_migrations");
        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].sql.contains("cake"));
    }

    #[test]
    fn test_empty_input() {
        let filtered = filter_protected_drops(vec![], "seaql_migrations");
        assert!(filtered.is_empty());
    }
}
