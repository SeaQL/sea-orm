use sea_orm::Statement;

/// Parse a list of SQL statements into hman-readable descriptions
pub fn summarize(stmts: &[Statement]) -> Vec<String> {
    stmts.iter().map(|s| describe(&s.sql)).collect()
}

fn describe(sql: &str) -> String {
    let upper = sql.to_uppercase();
    let sql = sql.trim();

    if upper.contains("CREATE SCHEMA") {
        if let Some(name) = extract_after(sql, &upper, "CREATE SCHEMA", Some("IF NOT EXISTS")) {
            return format!("Created schema: {name}");
        }
    }

    if upper.contains("CREATE TABLE") {
        if let Some(name) = extract_after(sql, &upper, "CREATE TABLE", Some("IF NOT EXISTS")) {
            return format!("Created table: {name}");
        }
    }

    if upper.contains("ALTER TABLE") {
        let table = extract_after(sql, &upper, "ALTER TABLE", None);
        if upper.contains("ADD COLUMN") {
            if let (Some(table), Some(col)) = (
                table.as_ref(),
                extract_after(sql, &upper, "ADD COLUMN", None),
            ) {
                return format!("Added column: {table}.{col}");
            }
        } else if upper.contains("RENAME COLUMN") {
            if let Some(table) = table {
                return format!("Renamed column on: {table}");
            }
        } else if upper.contains("DROP COLUMN") {
            if let (Some(table), Some(col)) = (
                table.as_ref(),
                extract_after(sql, &upper, "DROP COLUMN", None),
            ) {
                return format!("Dropped column: {table}.{col}");
            }
        } else if upper.contains("ADD CONSTRAINT") {
            if let Some(table) = table {
                return format!("Added foreign key on: {table}");
            }
        } else if upper.contains("DROP CONSTRAINT") {
            if let Some(table) = table {
                return format!("Dropped constraint on: {table}");
            }
        } else if upper.contains("DROP FOREIGN KEY") {
            if let Some(table) = table {
                return format!("Dropped foreign key on: {table}");
            }
        }
    }

    if upper.contains("ALTER TYPE") && upper.contains("ADD VALUE") {
        return "Added enum variant".to_string();
    }

    if upper.contains("DROP TABLE") {
        if let Some(name) = extract_after(sql, &upper, "DROP TABLE", Some("IF EXISTS")) {
            return format!("Dropped table: {name}");
        }
    }

    if upper.contains("CREATE INDEX") || upper.contains("CREATE UNIQUE INDEX") {
        if let Some(pos) = upper.find(" ON ") {
            let after = sql[pos + " ON ".len()..].trim_start();
            let table = extract_identifier(after);
            let kind = if upper.contains("UNIQUE") {
                "unique index"
            } else {
                "index"
            };
            return format!("Added {kind} on: {table}");
        }
    }

    if upper.contains("CREATE TYPE") {
        return "Created enum type".to_string();
    }

    // Fallback: first 80 chars of SQL
    if sql.len() > 80 {
        format!("SQL: {}...", &sql[..80])
    } else {
        format!("SQL: {sql}")
    }
}

fn extract_after(sql: &str, upper: &str, keyword: &str, skip: Option<&str>) -> Option<String> {
    let pos = upper.find(keyword)?;
    let rest = sql[pos + keyword.len()..].trim_start();
    let rest_upper = &upper[pos + keyword.len()..];
    let rest_upper = rest_upper.trim_start();
    let rest = if let Some(skip) = skip {
        if rest_upper.starts_with(skip) {
            rest[skip.len()..].trim_start()
        } else {
            rest
        }
    } else {
        rest
    };
    Some(extract_identifier(rest))
}

/// Extracts a (possibly schema-qualified) identifier, e.g. `"test"."person"`
/// becomes `test.person`, while a bare `"person"` stays `person`.
fn extract_identifier(s: &str) -> String {
    let (first, rest) = extract_one_identifier(s);
    if let Some(rest) = rest.strip_prefix('.') {
        let (second, _) = extract_one_identifier(rest);
        if !second.is_empty() {
            return format!("{first}.{second}");
        }
    }
    first
}

/// Extracts a single identifier token, returning it along with the unconsumed rest.
fn extract_one_identifier(s: &str) -> (String, &str) {
    let s = s.trim_start();
    if let Some(rest) = s.strip_prefix('"') {
        // Double-quoted identifier
        let end = rest.find('"').unwrap_or(rest.len());
        (rest[..end].to_string(), &rest[(end + 1).min(rest.len())..])
    } else if let Some(rest) = s.strip_prefix('`') {
        // Backtick-quoted identifier (MySQL)
        let end = rest.find('`').unwrap_or(rest.len());
        (rest[..end].to_string(), &rest[(end + 1).min(rest.len())..])
    } else {
        // Unquoted: take until whitespace, `(`, or `.`
        let end = s
            .find(|c: char| c.is_whitespace() || c == '(' || c == '.')
            .unwrap_or(s.len());
        (s[..end].to_string(), &s[end..])
    }
}
