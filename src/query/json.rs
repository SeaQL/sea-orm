use crate::{DbErr, FromQueryResult, QueryResult, QueryResultRow};
use serde_json::Map;
pub use serde_json::Value as JsonValue;

impl FromQueryResult for JsonValue {
    fn from_query_result(res: &QueryResult, pre: &str) -> Result<Self, DbErr> {
        match &res.row {
            #[cfg(feature = "sqlx-mysql")]
            QueryResultRow::SqlxMySql(row) => {
                use serde_json::json;
                use sqlx::{Column, MySql, Row, Type};
                let mut map = Map::new();
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
                                map.insert(
                                    col.to_owned(),
                                    json!(res.try_get::<Option<$type>>(pre, &col)?),
                                );
                                continue;
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
                }
                Ok(JsonValue::Object(map))
            }
            #[cfg(feature = "sqlx-postgres")]
            QueryResultRow::SqlxPostgres(row) => {
                use serde_json::json;
                use sqlx::{Column, Postgres, Row, Type};
                let mut map = Map::new();
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
                                map.insert(
                                    col.to_owned(),
                                    json!(res.try_get::<Option<$type>>(pre, &col)?),
                                );
                                continue;
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
                    match_postgres_type!(String);
                }
                Ok(JsonValue::Object(map))
            }
            #[cfg(feature = "sqlx-sqlite")]
            QueryResultRow::SqlxSqlite(row) => {
                use serde_json::json;
                use sqlx::{Column, Row, Sqlite, Type};
                let mut map = Map::new();
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
                                map.insert(
                                    col.to_owned(),
                                    json!(res.try_get::<Option<$type>>(pre, &col)?),
                                );
                                continue;
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
                    match_sqlite_type!(String);
                }
                Ok(JsonValue::Object(map))
            }
            #[cfg(feature = "mock")]
            QueryResultRow::Mock(row) => {
                let mut map = Map::new();
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
