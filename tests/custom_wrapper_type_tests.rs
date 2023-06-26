pub mod common;

pub use common::{
    features::{
        custom_wrapper::{Model, StringVec},
        *,
    },
    setup::*,
    TestContext,
};
use pretty_assertions::assert_eq;
use sea_orm::{entity::prelude::*, DatabaseConnection};
use sea_query::ValueType;

#[sea_orm_macros::test]
#[cfg(all(feature = "sqlx-postgres", feature = "postgres-array"))]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("custom_wrapper_tests").await;
    create_tables(&ctx.db).await?;
    insert_value(&ctx.db).await?;
    ctx.delete().await;

    Ok(())
}

pub async fn insert_value(_db: &DatabaseConnection) -> Result<(), DbErr> {
    assert_eq!(StringVec::type_name(), "StringVec");

    let model = Model {
        id: 1,
        str_vec: StringVec(vec!["ab".to_string(), "cd".to_string()]),
    };

    let string = Value::from(model.str_vec);
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

    Ok(())
}
