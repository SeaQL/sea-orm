#[allow(unused_imports)]
use crate::{ConnectionTrait, DbBackend, DbErr};
use sea_query::{TableCreateStatement, extension::postgres::TypeCreateStatement};

/// Stores the discovered schema from the database, including tables and enums
#[derive(Default)]
pub(crate) struct DiscoveredSchema {
    pub(crate) tables: Vec<TableCreateStatement>,
    pub(crate) enums: Vec<TypeCreateStatement>,
}

pub(crate) async fn discover_existing_schema<C>(db: &C) -> Result<DiscoveredSchema, DbErr>
where
    C: ConnectionTrait + sea_schema::Connection,
{
    //TODO: discover ONLY existing schema
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

            Ok(DiscoveredSchema {
                tables: schema.tables.iter().map(|table| table.write()).collect(),
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

            Ok(DiscoveredSchema {
                tables: schema.tables.iter().map(|table| table.write()).collect(),
                enums: schema.enums.iter().map(|def| def.write()).collect(),
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
