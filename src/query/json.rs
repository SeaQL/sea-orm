use crate::{FromQueryResult, QueryResult, QueryResultRow, TypeErr};
pub use serde_json::Value as JsonValue;
use serde_json::{json, Map};

impl FromQueryResult for JsonValue {
    fn from_query_result(res: &QueryResult, pre: &str) -> Result<Self, TypeErr> {
        match &res.row {
            QueryResultRow::SqlxMySql(row) => {
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
                                map.insert(col.to_owned(), json!(res.try_get::<Option<$type>>(pre, &col)?));
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
        }
    }
}
