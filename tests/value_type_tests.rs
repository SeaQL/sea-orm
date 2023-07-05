pub mod common;

use std::vec;

pub use common::{
    features::{
        value_type::{Boolbean, Integer, Model, StringVec},
        *,
    },
    setup::*,
    TestContext,
};
use pretty_assertions::assert_eq;
use sea_orm::{entity::prelude::*, entity::*, DatabaseConnection};
use sea_query::{ValueType, ValueTypeErr};

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("value_type_tests").await;
    create_tables(&ctx.db).await?;
    insert_value(&ctx.db).await?;
    ctx.delete().await;

    type_test();
    conversion_test();

    Ok(())
}

pub async fn insert_value(db: &DatabaseConnection) -> Result<(), DbErr> {
    let model = Model {
        id: 1,
        number: Integer(48),
    };
    let result = model.clone().into_active_model().insert(db).await?;
    assert_eq!(result, model);

    Ok(())
}

pub fn type_test() {
    assert_eq!(StringVec::type_name(), "StringVec");

    // self implied
    assert_eq!(Integer::array_type(), sea_orm::sea_query::ArrayType::Int);
    assert_eq!(Integer::array_type(), sea_orm::sea_query::ArrayType::Int);
    // custom types
    assert_eq!(
        Boolbean::column_type(),
        sea_orm::sea_query::ColumnType::Boolean
    );
    assert_eq!(
        Boolbean::array_type(),
        sea_orm::sea_query::ArrayType::String
    );
    assert_eq!(
        StringVec::column_type(),
        sea_orm::sea_query::ColumnType::String(Some(1))
    );
    assert_eq!(
        StringVec::array_type(),
        sea_orm::sea_query::ArrayType::String
    );
}

pub fn conversion_test() {
    let stringvec = StringVec(vec!["ab".to_string(), "cd".to_string()]);
    let string: Value = stringvec.into();
    assert_eq!(
        string,
        Value::Array(
            sea_query::ArrayType::String,
            Some(Box::new(vec![
                "ab".to_string().into(),
                "cd".to_string().into()
            ]))
        )
    );

    let value_random_int = sea_query::Value::Int(Some(523));
    let unwrap_int = Integer::unwrap(value_random_int.clone());
    let try_from_int =
        <Integer as ValueType>::try_from(value_random_int).expect("should be ok to convert");

    // tests for unwrap and try_from
    let direct_int = Integer(523);
    assert_eq!(direct_int, unwrap_int);
    assert_eq!(direct_int, try_from_int);

    // test for error
    let try_from_string_vec = <StringVec as ValueType>::try_from(Value::Char(Some('a')))
        .expect_err("should not be ok to convert char to stringvec");
    assert_eq!(try_from_string_vec.to_string(), ValueTypeErr.to_string());
}
