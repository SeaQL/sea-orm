#[allow(unused_imports)]
use crate::{ConnectionTrait, DbBackend, DbErr};
use sea_query::{TableCreateStatement, extension::postgres::TypeCreateStatement};

/// Stores the discovered schema from the database, including tables and enums
#[derive(Default)]
pub(crate) struct DiscoveredSchema {
    pub(crate) tables: Vec<TableCreateStatement>,
    pub(crate) enums: Vec<TypeCreateStatement>,
    /// Schemas (namespaces) referenced by a registered entity's `schema_name`
    /// that do not yet exist in the database. Postgres only — MySQL treats
    /// `schema_name` as a database name (creating one is out of scope for a
    /// table-sync tool) and SQLite has no schema namespaces at all.
    pub(crate) missing_schemas: Vec<String>,
}

/// Whether a PostgreSQL schema (namespace) already exists.
#[cfg(feature = "sqlx-postgres")]
async fn pg_schema_exists<C: ConnectionTrait>(db: &C, schema: &str) -> Result<bool, DbErr> {
    use sea_query::{Expr, ExprTrait};

    let row = db
        .query_one(
            sea_query::SelectStatement::new()
                .expr(Expr::cust("COUNT(*) > 0"))
                .from(("information_schema", "schemata"))
                .and_where(Expr::col("schema_name").eq(schema)),
        )
        .await?
        .ok_or_else(|| DbErr::RecordNotFound("Can't check schema existence".into()))?;
    row.try_get_by_index(0)
}

/// Every schema name visible via `information_schema.schemata`, minus
/// `system_schemas` and `excluded`.
///
/// Used to power orphan-table discovery (`allow_dangerous`) across every
/// namespace in the current Postgres database: a schema no longer referenced
/// by any registered entity (e.g. after a `schema_name` rename) would
/// otherwise never be looked at, so its now-orphaned tables could never be
/// reported. Postgres-only — see `discover()`'s doc comment for why this
/// doesn't extend to MySQL, where `schema_name` addresses a separate database
/// rather than a namespace inside the current one.
#[cfg(feature = "sqlx-postgres")]
async fn list_schemas_excluding<C: ConnectionTrait>(
    db: &C,
    system_schemas: &[&str],
    like_patterns: &[&str],
    excluded: &[String],
) -> Result<Vec<String>, DbErr> {
    use sea_query::{Alias, Condition, Expr, ExprTrait, Query};

    let mut cond = Condition::all();
    for sys in system_schemas {
        cond = cond.add(Expr::col("schema_name").ne(*sys));
    }
    for pattern in like_patterns {
        cond = cond.add(Expr::cust(format!("schema_name NOT LIKE '{pattern}'")));
    }
    for schema in excluded {
        cond = cond.add(Expr::col("schema_name").ne(schema.as_str()));
    }

    let rows = db
        .query_all(
            Query::select()
                .column(Alias::new("schema_name"))
                .from(("information_schema", "schemata"))
                .cond_where(cond),
        )
        .await?;

    rows.iter()
        .map(|row| row.try_get_by_index::<String>(0))
        .collect()
}

/// Every non-system PostgreSQL schema in the database, minus `excluded`.
/// Always skips the built-in `pg_catalog`, `information_schema`, `pg_toast`,
/// and temp schemas.
#[cfg(feature = "sqlx-postgres")]
pub(crate) async fn pg_list_all_schemas<C: ConnectionTrait>(
    db: &C,
    excluded: &[String],
) -> Result<Vec<String>, DbErr> {
    list_schemas_excluding(
        db,
        &["pg_catalog", "information_schema", "pg_toast"],
        &["pg_temp_%", "pg_toast_temp_%"],
        excluded,
    )
    .await
}

/// Re-point a `TableCreateStatement`'s table ref at an explicit schema, so it
/// matches the schema-qualified `TableRef` produced by `EntityName::table_ref`
/// for entities with `#[sea_orm(schema_name = "...")]`. Discovered tables come
/// back unqualified (bare table name) regardless of which schema they were
/// queried from, since sea-schema's writers only ever use `Alias::new(&name)`.
#[cfg(any(feature = "sqlx-postgres", feature = "sqlx-mysql"))]
fn qualify_table(mut stmt: TableCreateStatement, schema: &str) -> TableCreateStatement {
    use sea_query::{IntoTableRef, TableName, TableRef};

    if let Some(TableRef::Table(TableName(_, table), _)) = stmt.get_table_name().cloned() {
        stmt.table((schema.to_owned(), table).into_table_ref());
    }
    stmt
}

pub(crate) async fn discover_existing_schema<C>(
    db: &C,
    extra_schemas: &[String],
) -> Result<DiscoveredSchema, DbErr>
where
    C: ConnectionTrait + sea_schema::Connection,
{
    let _ = extra_schemas; // No drivers - no schemes
    match db.get_database_backend() {
        #[cfg(feature = "sqlx-mysql")]
        DbBackend::MySql => {
            use sea_schema::{mysql::discovery::SchemaDiscovery, probe::SchemaProbe};

            let current_schema: String = db
                .query_one(
                    sea_query::SelectStatement::new()
                        .expr(sea_schema::mysql::MySql::get_current_schema()),
                )
                .await?
                .ok_or_else(|| DbErr::RecordNotFound("Can't get current schema".into()))?
                .try_get_by_index(0)?;
            let schema_discovery = SchemaDiscovery::new_no_exec(&current_schema);

            let schema = schema_discovery
                .discover_with(db)
                .await
                .map_err(|err| DbErr::Query(crate::RuntimeErr::SqlxError(err.into())))?;

            // TODO: This multi-scheme discovery should be part of sea-schema instead
            let mut tables: Vec<TableCreateStatement> =
                schema.tables.iter().map(|table| table.write()).collect();

            for extra_schema in extra_schemas {
                if extra_schema == &current_schema {
                    continue;
                }
                let schema = SchemaDiscovery::new_no_exec(extra_schema)
                    .discover_with(db)
                    .await
                    .map_err(|err| DbErr::Query(crate::RuntimeErr::SqlxError(err.into())))?;
                tables.extend(
                    schema
                        .tables
                        .iter()
                        .map(|table| qualify_table(table.write(), extra_schema)),
                );
            }

            Ok(DiscoveredSchema {
                tables,
                enums: vec![],
                missing_schemas: vec![],
            })
        }
        #[cfg(feature = "sqlx-postgres")]
        DbBackend::Postgres => {
            use sea_schema::{postgres::discovery::SchemaDiscovery, probe::SchemaProbe};

            let current_schema: String = db
                .query_one(
                    sea_query::SelectStatement::new()
                        .expr(sea_schema::postgres::Postgres::get_current_schema()),
                )
                .await?
                .ok_or_else(|| DbErr::RecordNotFound("Can't get current schema".into()))?
                .try_get_by_index(0)?;
            let schema_discovery = SchemaDiscovery::new_no_exec(&current_schema);

            let schema = schema_discovery
                .discover_with(db)
                .await
                .map_err(|err| DbErr::Query(crate::RuntimeErr::SqlxError(err.into())))?;

            // TODO: This multi-scheme discovery should be part of sea-schema instead
            let mut tables: Vec<TableCreateStatement> =
                schema.tables.iter().map(|table| table.write()).collect();
            let mut enums: Vec<TypeCreateStatement> =
                schema.enums.iter().map(|def| def.write()).collect();
            let mut missing_schemas = Vec::new();

            for extra_schema in extra_schemas {
                if extra_schema == &current_schema {
                    continue;
                }
                if !pg_schema_exists(db, extra_schema).await? {
                    missing_schemas.push(extra_schema.clone());
                }
                let schema = SchemaDiscovery::new_no_exec(extra_schema)
                    .discover_with(db)
                    .await
                    .map_err(|err| DbErr::Query(crate::RuntimeErr::SqlxError(err.into())))?;
                tables.extend(
                    schema
                        .tables
                        .iter()
                        .map(|table| qualify_table(table.write(), extra_schema)),
                );
                enums.extend(schema.enums.iter().map(|def| def.write()));
            }

            Ok(DiscoveredSchema {
                tables,
                enums,
                missing_schemas,
            })
        }
        #[cfg(feature = "sqlx-sqlite")]
        DbBackend::Sqlite => {
            use sea_schema::sqlite::{SqliteDiscoveryError, discovery::SchemaDiscovery};
            let schema = SchemaDiscovery::discover_with(db)
                .await
                .map_err(|err| {
                    DbErr::Query(match err {
                        SqliteDiscoveryError::SqlxError(err) => {
                            crate::RuntimeErr::SqlxError(err.into())
                        }
                        _ => crate::RuntimeErr::Internal(format!("{err:?}")),
                    })
                })?
                .merge_indexes_into_table();
            Ok(DiscoveredSchema {
                tables: schema.tables.iter().map(|table| table.write()).collect(),
                enums: vec![],
                missing_schemas: vec![],
            })
        }
        #[cfg(feature = "rusqlite")]
        DbBackend::Sqlite => {
            use sea_schema::sqlite::{SqliteDiscoveryError, discovery::SchemaDiscovery};
            let schema = SchemaDiscovery::discover_with(db)
                .map_err(|err| {
                    DbErr::Query(match err {
                        SqliteDiscoveryError::RusqliteError(err) => {
                            crate::RuntimeErr::Rusqlite(err.into())
                        }
                        _ => crate::RuntimeErr::Internal(format!("{err:?}")),
                    })
                })?
                .merge_indexes_into_table();
            Ok(DiscoveredSchema {
                tables: schema.tables.iter().map(|table| table.write()).collect(),
                enums: vec![],
                missing_schemas: vec![],
            })
        }
        #[allow(unreachable_patterns)]
        other => Err(DbErr::BackendNotSupported {
            db: other.as_str(),
            ctx: "discover_existing_schema",
        }),
    }
}
