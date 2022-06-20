use sea_orm::entity::prelude::*;
use sea_orm::{TryGetError, TryGetable};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "json_vec")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub str_vec: StringVec,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StringVec(pub Vec<String>);

impl From<StringVec> for Value {
    fn from(source: StringVec) -> Self {
        Value::String(serde_json::to_string(&source).ok().map(|s| Box::new(s)))
    }
}

impl TryGetable for StringVec {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let json_str: String = res.try_get(pre, col).map_err(TryGetError::DbErr)?;
        serde_json::from_str(&json_str).map_err(|e| TryGetError::DbErr(DbErr::Json(e.to_string())))
    }
}

impl sea_query::ValueType for StringVec {
    fn try_from(v: Value) -> Result<Self, sea_query::ValueTypeErr> {
        match v {
            Value::String(Some(x)) => Ok(StringVec(
                serde_json::from_str(&x).map_err(|_| sea_query::ValueTypeErr)?,
            )),
            _ => Err(sea_query::ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(StringVec).to_owned()
    }

    fn column_type() -> sea_query::ColumnType {
        sea_query::ColumnType::String(None)
    }
}
