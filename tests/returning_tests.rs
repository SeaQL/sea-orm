pub mod common;

pub use common::{features::*, setup::*, TestContext};
use sea_orm::{entity::prelude::*, entity::*, DatabaseConnection};

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("returning_tests").await;
    let db = &ctx.db;

    match db {
        #[cfg(feature = "sqlx-mysql")]
        DatabaseConnection::SqlxMySqlPoolConnection { .. } => {
            let version = db.db_version();
            match version.as_str() {
                "5.7.26" => assert!(!db.db_support_returning()),
                _ => unimplemented!("Version {} is not included", version),
            };
        },
        #[cfg(feature = "sqlx-postgres")]
        DatabaseConnection::SqlxPostgresPoolConnection(_) => {
            assert!(db.db_support_returning());
        },
        #[cfg(feature = "sqlx-sqlite")]
        DatabaseConnection::SqlxSqlitePoolConnection(_) => {},
        _ => unreachable!(),
    }

    ctx.delete().await;

    Ok(())
}
