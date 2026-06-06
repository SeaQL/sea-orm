//! `DeriveValueType` over an inner type that does NOT impl `TryFromU64`
//! must still compile: the macro must not emit a spurious `TryFromU64`
//! delegation. `MyCustomInner` deliberately omits `TryFromU64`, so if
//! `DeriveValueType` tried to delegate to it this file would fail to build.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MyCustomInner(pub String);

impl From<MyCustomInner> for sea_orm::Value {
    fn from(v: MyCustomInner) -> Self {
        sea_orm::Value::String(Some(v.0))
    }
}

impl sea_orm::TryGetable for MyCustomInner {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        idx: I,
    ) -> Result<Self, sea_orm::TryGetError> {
        String::try_get_by(res, idx).map(MyCustomInner)
    }
}

impl sea_orm::sea_query::ValueType for MyCustomInner {
    fn try_from(v: sea_orm::Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
        <String as sea_orm::sea_query::ValueType>::try_from(v).map(MyCustomInner)
    }
    fn type_name() -> String {
        "MyCustomInner".to_owned()
    }
    fn array_type() -> sea_orm::sea_query::ArrayType {
        sea_orm::sea_query::ArrayType::String
    }
    fn column_type() -> sea_orm::sea_query::ColumnType {
        sea_orm::sea_query::ColumnType::Text
    }
}

impl sea_orm::sea_query::Nullable for MyCustomInner {
    fn null() -> sea_orm::Value {
        sea_orm::Value::String(None)
    }
}

// Deliberately NO `impl TryFromU64 for MyCustomInner`. If the
// `DeriveValueType` macro tries to delegate to it, this file won't compile.
#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType)]
pub struct Wrap(pub MyCustomInner);

#[test]
fn wrap_over_non_tryfromu64_inner_compiles() {
    // The fact that this file compiles at all is the assertion.
    let _ = Wrap(MyCustomInner("hi".into()));
}
