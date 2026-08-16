#[allow(unused_imports)]
use crate::{ConnectionTrait, DbBackend, DbErr};
use sea_query::{TableCreateStatement, extension::postgres::TypeCreateStatement};

/// Stores the discovered schema from the database, including tables and enums
#[derive(Default)]
pub(crate) struct DiscoveredSchema {
    pub(crate) tables: Vec<TableCreateStatement>,
    pub(crate) enums: Vec<TypeCreateStatement>,
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
                enums.extend(schema.enums.iter().map(|def| def.write()));
            }

            Ok(DiscoveredSchema { tables, enums })
        }
        #[cfg(feature = "sqlx-sqlite")]
        DbBackend::Sqlite => {
            use sea_schema::sqlite::{SqliteDiscoveryError, discovery::SchemaDiscovery};
            let _ = extra_schemas; // Doesn't have schemes
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
            })
        }
        #[allow(unreachable_patterns)]
        other => Err(DbErr::BackendNotSupported {
            db: other.as_str(),
            ctx: "discover_existing_schema",
        }),
    }
}
