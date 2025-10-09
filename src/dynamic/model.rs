use crate::{DbErr, QueryResult};
use sea_query::{ArrayType, DynIden, Value};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelType {
    pub fields: Vec<FieldType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldType {
    pub(super) field: Arc<str>,
    pub(super) type_: ArrayType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub fields: Vec<FieldValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldValue {
    pub(super) field: Arc<str>,
    pub value: Value,
}

impl FieldType {
    pub fn new(iden: DynIden, type_: ArrayType) -> Self {
        Self {
            field: Arc::from(iden.inner()),
            type_,
        }
    }

    pub fn field(&self) -> &str {
        &self.field
    }
}

impl FieldValue {
    pub fn field(&self) -> &str {
        &self.field
    }
}

impl ModelType {
    pub fn from_query_result(&self, res: &QueryResult, pre: &str) -> Result<Model, DbErr> {
        let mut fields = Vec::new();
        for f in self.fields.iter() {
            fields.push(FieldValue {
                field: f.field.clone(),
                value: try_get(res, pre, f.field(), &f.type_)?,
            });
        }
        Ok(Model { fields })
    }
}

impl Model {
    pub fn try_get(&self, col: &str) -> Result<&Value, DbErr> {
        for field in &self.fields {
            if field.field() == col {
                return Ok(&field.value);
            }
        }
        Err(DbErr::Type(format!("{col} not exist")))
    }
}

fn try_get(res: &QueryResult, pre: &str, col: &str, ty: &ArrayType) -> Result<Value, DbErr> {
    // how to handle postgres-array?
    Ok(match ty {
        ArrayType::Bool => Value::Bool(res.try_get(pre, col)?),
        ArrayType::TinyInt => Value::TinyInt(res.try_get(pre, col)?),
        ArrayType::SmallInt => Value::SmallInt(res.try_get(pre, col)?),
        ArrayType::Int => Value::Int(res.try_get(pre, col)?),
        ArrayType::BigInt => Value::BigInt(res.try_get(pre, col)?),
        ArrayType::TinyUnsigned => Value::TinyUnsigned(res.try_get(pre, col)?),
        ArrayType::SmallUnsigned => Value::SmallUnsigned(res.try_get(pre, col)?),
        ArrayType::Unsigned => Value::Unsigned(res.try_get(pre, col)?),
        ArrayType::BigUnsigned => Value::BigUnsigned(res.try_get(pre, col)?),
        ArrayType::Float => Value::Float(res.try_get(pre, col)?),
        ArrayType::Double => Value::Double(res.try_get(pre, col)?),
        ArrayType::String => Value::String(res.try_get(pre, col)?),
        ArrayType::Char => return Err(DbErr::Type("Unsupported type: char".into())),
        ArrayType::Bytes => Value::Bytes(res.try_get(pre, col)?),

        #[cfg(feature = "with-json")]
        ArrayType::Json => Value::Json(res.try_get(pre, col)?),

        #[cfg(feature = "with-chrono")]
        ArrayType::ChronoDate => Value::ChronoDate(res.try_get(pre, col)?),

        #[cfg(feature = "with-chrono")]
        ArrayType::ChronoTime => Value::ChronoTime(res.try_get(pre, col)?),

        #[cfg(feature = "with-chrono")]
        ArrayType::ChronoDateTime => Value::ChronoDateTime(res.try_get(pre, col)?),

        #[cfg(feature = "with-chrono")]
        ArrayType::ChronoDateTimeUtc => Value::ChronoDateTimeUtc(res.try_get(pre, col)?),

        #[cfg(feature = "with-chrono")]
        ArrayType::ChronoDateTimeLocal => Value::ChronoDateTimeLocal(res.try_get(pre, col)?),

        #[cfg(feature = "with-chrono")]
        ArrayType::ChronoDateTimeWithTimeZone => {
            Value::ChronoDateTimeWithTimeZone(res.try_get(pre, col)?)
        }

        #[cfg(feature = "with-time")]
        ArrayType::TimeDate => Value::TimeDate(res.try_get(pre, col)?),

        #[cfg(feature = "with-time")]
        ArrayType::TimeTime => Value::TimeTime(res.try_get(pre, col)?),

        #[cfg(feature = "with-time")]
        ArrayType::TimeDateTime => Value::TimeDateTime(res.try_get(pre, col)?),

        #[cfg(feature = "with-time")]
        ArrayType::TimeDateTimeWithTimeZone => {
            Value::TimeDateTimeWithTimeZone(res.try_get(pre, col)?)
        }

        #[cfg(feature = "with-uuid")]
        ArrayType::Uuid => Value::Uuid(res.try_get(pre, col)?),

        #[cfg(feature = "with-rust_decimal")]
        ArrayType::Decimal => Value::Decimal(res.try_get(pre, col)?),

        #[cfg(feature = "with-bigdecimal")]
        ArrayType::BigDecimal => {
            Value::BigDecimal(res.try_get::<Option<_>>(pre, col)?.map(Box::new))
        }

        #[cfg(feature = "postgres-vector")]
        ArrayType::Vector => Value::Vector(res.try_get(pre, col)?),

        #[cfg(feature = "with-ipnetwork")]
        ArrayType::IpNetwork => Value::IpNetwork(res.try_get(pre, col)?),
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{QueryResultRow, database::IntoMockRow, dynamic::Entity};

    #[test]
    fn test_from_query_result() {
        let result = QueryResult {
            row: QueryResultRow::Mock(
                crate::tests_cfg::cake::Model {
                    id: 12,
                    name: "hello".into(),
                }
                .into_mock_row(),
            ),
        };
        let model_ty = Entity::from_entity(crate::tests_cfg::cake::Entity).to_model_type();
        assert_eq!(
            model_ty,
            ModelType {
                fields: vec![
                    FieldType {
                        field: Arc::from("id"),
                        type_: ArrayType::Int,
                    },
                    FieldType {
                        field: Arc::from("name"),
                        type_: ArrayType::String,
                    },
                ],
            }
        );
        assert_eq!(
            model_ty.from_query_result(&result, "").unwrap(),
            Model {
                fields: vec![
                    FieldValue {
                        field: Arc::from("id"),
                        value: 12i32.into(),
                    },
                    FieldValue {
                        field: Arc::from("name"),
                        value: "hello".into(),
                    }
                ],
            }
        );
    }
}
