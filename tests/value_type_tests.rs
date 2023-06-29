pub mod common;

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
use sea_query::ValueType;

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

    Ok(())
}

pub async fn insert_value(db: &DatabaseConnection) -> Result<(), DbErr> {
    assert_eq!(StringVec::type_name(), "StringVec");

    let stringvec = StringVec(vec!["ab".to_string(), "cd".to_string()]);
    let string = Value::from(stringvec);
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

    assert_eq!(
        Boolbean::column_type(),
        sea_orm::sea_query::ColumnType::Boolean
    );
    assert_eq!(Boolbean::array_type(), sea_orm::sea_query::ArrayType::Bool);
    assert_eq!(
        StringVec::column_type(),
        sea_orm::sea_query::ColumnType::String(Some(1))
    );
    assert_eq!(
        StringVec::array_type(),
        sea_orm::sea_query::ArrayType::String
    );

    let random_testing_int = 523;
    let value_random_testing_int = sea_query::Value::Int(Some(523));

    let direct_int = Integer(random_testing_int);
    let unwrap_int = Integer::unwrap(value_random_testing_int);

    assert_eq!(direct_int, unwrap_int);

    let model = Model {
        id: 1,
        number: Integer(48),
    };

    let result = model.clone().into_active_model().insert(db).await?;

    assert_eq!(result, model);

    Ok(())
}
