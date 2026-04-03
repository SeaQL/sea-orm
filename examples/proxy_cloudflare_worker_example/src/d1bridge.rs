use std::{collections::BTreeMap, sync::Arc};
use wasm_bindgen::JsValue;

use sea_orm::{
    Database, DatabaseConnection, DbBackend, DbErr, ProxyDatabaseTrait, ProxyExecResult, ProxyRow,
    RuntimeErr, Statement, Value, Values,
};
use worker::{D1Database, console_log};

struct D1(Arc<D1Database>);

pub async fn connect_d1(d1: D1Database) -> Result<DatabaseConnection, DbErr> {
    Database::connect_proxy(DbBackend::Sqlite, Arc::new(Box::new(D1(d1.into())))).await
}

impl std::fmt::Debug for D1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "D1Stuct")
    }
}

#[async_trait::async_trait]
impl ProxyDatabaseTrait for D1 {
    async fn query(&self, statement: Statement) -> Result<Vec<ProxyRow>, DbErr> {
        let d1 = Arc::clone(&self.0);
        let values = map_values(&statement);
        let sql = statement.sql;

        worker::send::SendFuture::new(async move {
            let res = d1.prepare(sql).bind(&values)?.all().await?;

            if let Some(message) = res.error() {
                anyhow::bail!(message.to_string());
            }

            let rows = res.results::<serde_json::Value>()?;
            anyhow::Ok(rows.into_iter().map(json_to_proxy_row).collect())
        })
        .await
        .map_err(|e| DbErr::Exec(RuntimeErr::Internal(e.to_string())))
    }

    async fn execute(&self, statement: Statement) -> Result<ProxyExecResult, DbErr> {
        let d1 = Arc::clone(&self.0);
        let values = map_values(&statement);
        let sql = statement.sql;

        worker::send::SendFuture::new(async move {
            let meta = d1.prepare(sql).bind(&values)?.run().await?.meta()?;

            let last_insert_id = meta.as_ref().and_then(|m| m.last_row_id).unwrap_or(0) as u64;
            let rows_affected = meta.and_then(|m| m.rows_written).unwrap_or(0) as u64;

            anyhow::Ok(ProxyExecResult {
                last_insert_id,
                rows_affected,
            })
        })
        .await
        .map_err(|err| DbErr::Conn(RuntimeErr::Internal(err.to_string())))
    }
}

fn map_values(statement: &Statement) -> Vec<JsValue> {
    match &statement.values {
        Some(Values(values)) => values
            .iter()
            .map(|val| match val {
                Value::Bool(Some(val)) => JsValue::from(*val),
                Value::Char(Some(val)) => JsValue::from(val.to_string()),

                // Float values.
                Value::Float(Some(val)) => JsValue::from_f64(*val as f64),
                Value::Double(Some(val)) => JsValue::from_f64(*val),

                // Signed values
                Value::BigInt(Some(val)) => JsValue::from(val.to_string()),
                Value::Int(Some(val)) => JsValue::from(*val),
                Value::SmallInt(Some(val)) => JsValue::from(*val),
                Value::TinyInt(Some(val)) => JsValue::from(*val),

                // Unsigned values
                Value::BigUnsigned(Some(val)) => JsValue::from(val.to_string()),
                Value::Unsigned(Some(val)) => JsValue::from(*val),
                Value::SmallUnsigned(Some(val)) => JsValue::from(*val),
                Value::TinyUnsigned(Some(val)) => JsValue::from(*val),

                Value::String(Some(val)) => JsValue::from(val.to_string()),
                Value::Json(Some(val)) => JsValue::from(val.to_string()),
                Value::Bytes(Some(val)) => JsValue::from(format!(
                    "X'{}'",
                    val.iter()
                        .map(|byte| format!("{:02x}", byte))
                        .collect::<String>()
                )),

                Value::ChronoDate(Some(val)) => JsValue::from(val.to_string()),
                Value::ChronoDateTime(Some(val)) => JsValue::from(val.to_string()),
                Value::ChronoDateTimeLocal(Some(val)) => JsValue::from(val.to_string()),
                Value::ChronoDateTimeUtc(Some(val)) => JsValue::from(val.to_string()),
                Value::ChronoDateTimeWithTimeZone(Some(val)) => JsValue::from(val.to_string()),

                e => {
                    console_log!("running: {}", e);
                    JsValue::NULL
                }
            })
            .collect(),
        None => Vec::new(),
    }
}

fn json_to_proxy_row(row: serde_json::Value) -> ProxyRow {
    let mut values = BTreeMap::new();

    let Some(obj) = row.as_object() else {
        return ProxyRow { values };
    };

    for (k, v) in obj {
        let sea_val = match v {
            serde_json::Value::Bool(val) => Value::Bool(Some(*val)),
            serde_json::Value::Number(val) => {
                if let Some(i) = val.as_i64() {
                    Value::BigInt(Some(i))
                } else if let Some(u) = val.as_u64() {
                    Value::BigUnsigned(Some(u))
                } else {
                    Value::Double(Some(val.as_f64().unwrap_or(0.0)))
                }
            }
            serde_json::Value::String(val) => Value::String(Some(Box::new(val.clone()))),
            _ => unreachable!(),
        };
        values.insert(k.clone(), sea_val);
    }
    ProxyRow { values }
}
