#![cfg(all(feature = "sqlx-sqlite", feature = "with-json"))]
#![allow(unused_imports, dead_code)]

#[cfg(feature = "runtime-async-std")]
#[macro_export]
macro_rules! block_on {
    ($($expr:tt)*) => {
        ::async_std::task::block_on($($expr)*)
    };
}

#[cfg(feature = "runtime-tokio")]
#[macro_export]
macro_rules! block_on {
    ($($expr:tt)*) => {
        ::tokio::runtime::Runtime::new().unwrap().block_on($($expr)*)
    };
}

use pretty_assertions::assert_eq;
use sea_orm::{
    ColumnTrait, ConnectionTrait, Database, DbErr, EntityTrait, FromQueryResult, JsonValue,
    QueryFilter, QueryOrder, QuerySelect, Statement, Value,
};
use serde_json::json;

mod invitation_apply {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "invitation_apply")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub invitation_id: i32,
        pub status: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

#[sea_orm_macros::test]
async fn main() -> Result<(), DbErr> {
    dotenv::from_filename(".env.local").ok();
    dotenv::from_filename(".env").ok();

    let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_owned());
    let db = Database::connect(&base_url).await?;

    db
        .execute_unprepared("DROP TABLE IF EXISTS invitation_apply")
        .await?;
    db
        .execute_unprepared(
            "CREATE TABLE invitation_apply (\
            id INTEGER PRIMARY KEY AUTOINCREMENT,\
            invitation_id INTEGER NOT NULL,\
            status TEXT NOT NULL\
            )",
        )
        .await?;
    db
        .execute_unprepared(
            "INSERT INTO invitation_apply (invitation_id, status) VALUES \
            (1, 'Pending'),\
            (1, 'Pending'),\
            (1, 'Accepted'),\
            (2, 'Pending')",
        )
        .await?;

    let db_backend = db.get_database_backend();
    let got_raw = JsonValue::find_by_statement(Statement::from_sql_and_values(
        db_backend,
        "SELECT status, COUNT(status) AS count \
        FROM invitation_apply \
        WHERE invitation_id = ? \
        GROUP BY status \
        ORDER BY status",
        [Value::Int(Some(1))],
    ))
    .all(&db)
    .await?;
    assert_eq!(
        got_raw,
        vec![
            json!({"status": "Accepted", "count": 1}),
            json!({"status": "Pending", "count": 2}),
        ]
    );

    let got_qb = invitation_apply::Entity::find()
        .select_only()
        .column(invitation_apply::Column::Status)
        .column_as(invitation_apply::Column::Status.count(), "count")
        .filter(invitation_apply::Column::InvitationId.eq(1))
        .group_by(invitation_apply::Column::Status)
        .order_by_asc(invitation_apply::Column::Status)
        .into_json()
        .all(&db)
        .await?;
    assert_eq!(
        got_qb,
        vec![
            json!({"status": "Accepted", "count": 1}),
            json!({"status": "Pending", "count": 2}),
        ]
    );

    Ok(())
}
