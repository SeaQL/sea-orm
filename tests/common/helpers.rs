use sea_orm::{DatabaseConnection, DbBackend, DbErr, query::*};
use sea_orm::sea_query::{Alias, Condition, Expr, Query};

#[cfg(feature = "schema-sync")]
use sea_orm::{InterpretConfig, InterpretResult, schema::SchemaBuilder};

/// Runs `discover` + `interpret_changes` + executes every emitted statement, then
/// returns the [`InterpretResult`] so callers can make additional assertions on
/// warnings / suggestions.
///
/// This is the "ground-truth" form of a sync round-trip: unlike tests that only
/// inspect the generated SQL, this helper actually applies the statements to the
/// live database so follow-up `column_exists`/`table_exists` checks are meaningful.
#[cfg(feature = "schema-sync")]
pub async fn discover_interpret_and_apply(
    db: &DatabaseConnection,
    builder: SchemaBuilder,
    config: InterpretConfig,
) -> Result<InterpretResult, DbErr> {
    let dangerous = config.allow_dangerous;
    let change_set = builder.discover(db, dangerous).await?;
    let result = sea_orm::interpret_changes(change_set, &config);
    for (_, stmt) in &result.statements {
        db.execute_raw(stmt.clone()).await?;
    }
    Ok(result)
}

pub async fn table_exists(db: &DatabaseConnection, table: &str) -> Result<bool, DbErr> {
    match db.get_database_backend() {
        #[cfg(feature = "sqlx-postgres")]
        DbBackend::Postgres => {
            let row = db
                .query_one(
                    Query::select()
                        .expr(Expr::cust("COUNT(*) > 0"))
                        .from((Alias::new("information_schema"), Alias::new("tables")))
                        .cond_where(
                            Condition::all()
                                .add(Expr::cust("table_schema = CURRENT_SCHEMA()"))
                                .add(Expr::col("table_name").eq(table)),
                        ),
                )
                .await?;
            Ok(row
                .map(|r| r.try_get_by_index::<bool>(0).unwrap_or(false))
                .unwrap_or(false))
        }
        #[cfg(feature = "sqlx-mysql")]
        DbBackend::MySql => {
            let row = db
                .query_one(
                    Query::select()
                        .expr(Expr::cust("COUNT(*) > 0"))
                        .from((Alias::new("information_schema"), Alias::new("tables")))
                        .cond_where(
                            Condition::all()
                                .add(Expr::cust("table_schema = DATABASE()"))
                                .add(Expr::col("table_name").eq(table)),
                        ),
                )
                .await?;
            Ok(row
                .map(|r| r.try_get_by_index::<bool>(0).unwrap_or(false))
                .unwrap_or(false))
        }
        #[cfg(any(feature = "sqlx-sqlite", feature = "rusqlite"))]
        DbBackend::Sqlite => {
            let row = db
                .query_one(
                    Query::select()
                        .expr(Expr::cust("COUNT(*) > 0"))
                        .from(Alias::new("sqlite_master"))
                        .cond_where(
                            Condition::all()
                                .add(Expr::col("type").eq("table"))
                                .add(Expr::col("name").eq(table)),
                        ),
                )
                .await?;
            Ok(row
                .map(|r| r.try_get_by_index::<bool>(0).unwrap_or(false))
                .unwrap_or(false))
        }
        _ => Ok(false),
    }
}

pub async fn column_exists(
    db: &DatabaseConnection,
    table: &str,
    column: &str,
) -> Result<bool, DbErr> {
    match db.get_database_backend() {
        #[cfg(feature = "sqlx-postgres")]
        DbBackend::Postgres => {
            let row = db
                .query_one(
                    Query::select()
                        .expr(Expr::cust("COUNT(*) > 0"))
                        .from((Alias::new("information_schema"), Alias::new("columns")))
                        .cond_where(
                            Condition::all()
                                .add(Expr::cust("table_schema = CURRENT_SCHEMA()"))
                                .add(Expr::col("table_name").eq(table))
                                .add(Expr::col("column_name").eq(column)),
                        ),
                )
                .await?;
            Ok(row
                .map(|r| r.try_get_by_index::<bool>(0).unwrap_or(false))
                .unwrap_or(false))
        }
        #[cfg(feature = "sqlx-mysql")]
        DbBackend::MySql => {
            let row = db
                .query_one(
                    Query::select()
                        .expr(Expr::cust("COUNT(*) > 0"))
                        .from((Alias::new("information_schema"), Alias::new("columns")))
                        .cond_where(
                            Condition::all()
                                .add(Expr::cust("table_schema = DATABASE()"))
                                .add(Expr::col("table_name").eq(table))
                                .add(Expr::col("column_name").eq(column)),
                        ),
                )
                .await?;
            Ok(row
                .map(|r| r.try_get_by_index::<bool>(0).unwrap_or(false))
                .unwrap_or(false))
        }
        #[cfg(any(feature = "sqlx-sqlite", feature = "rusqlite"))]
        DbBackend::Sqlite => {
            let rows = db
                .query_all_raw(sea_orm::Statement::from_string(
                    DbBackend::Sqlite,
                    format!("PRAGMA table_info(\"{table}\")"),
                ))
                .await?;
            Ok(rows.iter().any(|r| {
                r.try_get_by_index::<String>(1)
                    .map(|n| n == column)
                    .unwrap_or(false)
            }))
        }
        _ => Ok(false),
    }
}
