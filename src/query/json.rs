use crate::{FromQueryResult, QueryResult, QueryResultRow, TypeErr};
use serde_json::Map;
pub use serde_json::Value as JsonValue;

impl FromQueryResult for JsonValue {
    fn from_query_result(res: &QueryResult, pre: &str) -> Result<Self, TypeErr> {
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
    use crate::{entity::*, MockDatabase};
    use sea_query::Value;

    #[async_std::test]
    async fn to_json_1() {
        let db = MockDatabase::new()
            .append_query_results(vec![vec![maplit::btreemap! {
                "id" => Into::<Value>::into(128), "name" => Into::<Value>::into("apple")
            }
            .into()]])
            .into_database();

        assert_eq!(
            cake::Entity::find().into_json().one(&db).await.unwrap(),
            Some(serde_json::json!({
                "id": 128,
                "name": "apple"
            }))
        );
    }
}
