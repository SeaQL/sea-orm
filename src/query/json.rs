use crate::{DbErr, FromQueryResult, QueryResult, QueryResultRow};
use serde_json::Map;
pub use serde_json::Value as JsonValue;

impl FromQueryResult for JsonValue {
    fn from_query_result(res: &QueryResult, pre: &str) -> Result<Self, DbErr> {
        let mut map = Map::new();
        #[allow(unused_macros)]
        macro_rules! try_get_type {
            ( $type: ty, $col: ident ) => {
                if let Ok(v) = res.try_get::<Option<$type>>(pre, &$col) {
                    map.insert($col.to_owned(), json!(v));
                    continue;
                }
            };
        }
        match &res.row {
            #[cfg(feature = "sqlx-mysql")]
            QueryResultRow::SqlxMySql(row) => {
                use sea_connection::sqlx;
                use serde_json::json;
                use sqlx::{Column, MySql, Row, Type};
                for column in row.columns() {
                    let col = if !column.name().starts_with(pre) {
                        continue;
                    } else {
                        column.name().replacen(pre, "", 1)
                    };
                    let col_type = column.type_info();
                    macro_rules! match_mysql_type {
                        ( $type: ty ) => {
                            if <$type as Type<MySql>>::type_info().eq(col_type) {
                                try_get_type!($type, col)
                            }
                        };
                    }
                    match_mysql_type!(bool);
                    match_mysql_type!(i8);
                    match_mysql_type!(i16);
                    match_mysql_type!(i32);
                    match_mysql_type!(i64);
                    match_mysql_type!(u8);
                    match_mysql_type!(u16);
                    match_mysql_type!(u32);
                    match_mysql_type!(u64);
                    match_mysql_type!(f32);
                    match_mysql_type!(f64);
                    match_mysql_type!(String);
                    #[cfg(feature = "with-chrono")]
                    match_mysql_type!(chrono::NaiveDate);
                    #[cfg(feature = "with-chrono")]
                    match_mysql_type!(chrono::NaiveTime);
                    #[cfg(feature = "with-chrono")]
                    match_mysql_type!(chrono::NaiveDateTime);
                    #[cfg(feature = "with-chrono")]
                    match_mysql_type!(chrono::DateTime<chrono::Utc>);
                    #[cfg(feature = "with-rust_decimal")]
                    match_mysql_type!(rust_decimal::Decimal);
                    #[cfg(feature = "with-json")]
                    try_get_type!(serde_json::Value, col);
                    try_get_type!(String, col);
                    #[cfg(feature = "with-uuid")]
                    try_get_type!(uuid::Uuid, col);
                    try_get_type!(Vec<u8>, col);
                }
                Ok(JsonValue::Object(map))
            }
            #[cfg(feature = "sqlx-postgres")]
            QueryResultRow::SqlxPostgres(row) => {
                use sea_connection::sqlx;
                use serde_json::json;
                use sqlx::{Column, Postgres, Row, Type};
                for column in row.columns() {
                    let col = if !column.name().starts_with(pre) {
                        continue;
                    } else {
                        column.name().replacen(pre, "", 1)
                    };
                    let col_type = column.type_info();
                    macro_rules! match_postgres_type {
                        ( $type: ty ) => {
                            if <$type as Type<Postgres>>::type_info().eq(col_type) {
                                try_get_type!($type, col)
                            }
                        };
                    }
                    match_postgres_type!(bool);
                    match_postgres_type!(i8);
                    match_postgres_type!(i16);
                    match_postgres_type!(i32);
                    match_postgres_type!(i64);
                    // match_postgres_type!(u8); // unsupported by SQLx Postgres
                    // match_postgres_type!(u16); // unsupported by SQLx Postgres
                    match_postgres_type!(u32);
                    // match_postgres_type!(u64); // unsupported by SQLx Postgres
                    match_postgres_type!(f32);
                    match_postgres_type!(f64);
                    #[cfg(feature = "with-chrono")]
                    match_postgres_type!(chrono::NaiveDate);
                    #[cfg(feature = "with-chrono")]
                    match_postgres_type!(chrono::NaiveTime);
                    #[cfg(feature = "with-chrono")]
                    match_postgres_type!(chrono::NaiveDateTime);
                    #[cfg(feature = "with-chrono")]
                    match_postgres_type!(chrono::DateTime<chrono::FixedOffset>);
                    #[cfg(feature = "with-rust_decimal")]
                    match_postgres_type!(rust_decimal::Decimal);
                    #[cfg(feature = "with-json")]
                    try_get_type!(serde_json::Value, col);
                    try_get_type!(String, col);
                    #[cfg(feature = "with-uuid")]
                    try_get_type!(uuid::Uuid, col);
                    try_get_type!(Vec<u8>, col);
                }
                Ok(JsonValue::Object(map))
            }
            #[cfg(feature = "sqlx-sqlite")]
            QueryResultRow::SqlxSqlite(row) => {
                use sea_connection::sqlx;
                use serde_json::json;
                use sqlx::{Column, Row, Sqlite, Type};
                for column in row.columns() {
                    let col = if !column.name().starts_with(pre) {
                        continue;
                    } else {
                        column.name().replacen(pre, "", 1)
                    };
                    let col_type = column.type_info();
                    macro_rules! match_sqlite_type {
                        ( $type: ty ) => {
                            if <$type as Type<Sqlite>>::type_info().eq(col_type) {
                                try_get_type!($type, col)
                            }
                        };
                    }
                    match_sqlite_type!(bool);
                    match_sqlite_type!(i8);
                    match_sqlite_type!(i16);
                    match_sqlite_type!(i32);
                    match_sqlite_type!(i64);
                    match_sqlite_type!(u8);
                    match_sqlite_type!(u16);
                    match_sqlite_type!(u32);
                    // match_sqlite_type!(u64); // unsupported by SQLx Sqlite
                    match_sqlite_type!(f32);
                    match_sqlite_type!(f64);
                    #[cfg(feature = "with-chrono")]
                    match_sqlite_type!(chrono::NaiveDate);
                    #[cfg(feature = "with-chrono")]
                    match_sqlite_type!(chrono::NaiveTime);
                    #[cfg(feature = "with-chrono")]
                    match_sqlite_type!(chrono::NaiveDateTime);
                    try_get_type!(String, col);
                    #[cfg(feature = "with-uuid")]
                    try_get_type!(uuid::Uuid, col);
                    try_get_type!(Vec<u8>, col);
                }
                Ok(JsonValue::Object(map))
            }
            #[cfg(feature = "mock")]
            QueryResultRow::Mock(row) => {
                for (column, value) in row.clone().into_column_value_tuples() {
                    let col = if !column.starts_with(pre) {
                        continue;
                    } else {
                        column.replacen(pre, "", 1)
                    };
                    map.insert(col, sea_query::sea_value_to_json_value(&value));
                }
                Ok(JsonValue::Object(map))
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "mock")]
mod tests {
    use crate::tests_cfg::cake;
    use crate::{entity::*, DbBackend, DbErr, MockDatabase};
    use sea_query::Value;

    #[smol_potat::test]
    async fn to_json_1() -> Result<(), DbErr> {
        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results(vec![vec![maplit::btreemap! {
                "id" => Into::<Value>::into(128), "name" => Into::<Value>::into("apple")
            }]])
            .into_connection();

        assert_eq!(
            cake::Entity::find().into_json().one(&db).await.unwrap(),
            Some(serde_json::json!({
                "id": 128,
                "name": "apple"
            }))
        );

        Ok(())
    }
}
