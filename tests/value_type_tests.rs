#![allow(unused_imports, dead_code)]

pub mod common;

use std::sync::Arc;
use std::vec;

pub use common::{
    TestContext,
    features::{
        value_type::{
            MyInteger, StringVec, Tag1, Tag2, value_type_general, value_type_pg, value_type_pk,
        },
        *,
    },
    setup::*,
};
use pretty_assertions::assert_eq;
use sea_orm::{
    DatabaseConnection, DbBackend, QuerySelect,
    entity::{prelude::*, *},
};
use sea_query::{ArrayType, ColumnType, PostgresQueryBuilder, Value, ValueType, ValueTypeErr};

#[sea_orm_macros::test]
async fn main() -> Result<(), DbErr> {
    type_test();
    conversion_test();

    let ctx = TestContext::new("value_type_tests").await;

    create_value_type_table(&ctx.db).await?;
    insert_value_general(&ctx.db).await?;
    insert_value_pk(&ctx.db).await?;

    if cfg!(feature = "sqlx-postgres") {
        create_value_type_postgres_table(&ctx.db).await?;
        insert_value_postgres(&ctx.db).await?;
    }

    ctx.delete().await;

    Ok(())
}

pub async fn insert_value_general(db: &DatabaseConnection) -> Result<(), DbErr> {
    let model = value_type_general::Model {
        id: 1,
        number: 48.into(),
        tag_1: Tag1::Hard,
        tag_2: Tag2::Grey,
    };
    let result = model.clone().into_active_model().insert(db).await?;
    assert_eq!(result, model);

    Ok(())
}

pub async fn insert_value_pk(db: &DatabaseConnection) -> Result<(), DbErr> {
    let model = value_type_pk::Model {
        id: MyInteger(1),
        val: MyInteger(2),
    };
    let result = model.clone().into_active_model().insert(db).await?;
    assert_eq!(result, model);

    let mut model = result.into_active_model();
    model.val = Set(MyInteger(3));
    model.save(db).await?;
    assert_eq!(
        value_type_pk::Entity::find_by_id(MyInteger(1))
            .one(db)
            .await?
            .unwrap()
            .val,
        MyInteger(3)
    );

    Ok(())
}

pub async fn insert_value_postgres(db: &DatabaseConnection) -> Result<(), DbErr> {
    let model = value_type_pg::Model {
        id: 1,
        number: 48.into(),
        str_vec: StringVec(vec!["ab".to_string(), "cd".to_string()]),
    };
    let result = model.clone().into_active_model().insert(db).await?;
    assert_eq!(result, model);

    let query = sea_query::Query::select()
        .from(value_type_pg::Entity)
        .column((value_type_pg::Entity, value_type_pg::Column::Number))
        .and_where(value_type_pg::Column::Id.eq(1))
        .take();

    let row = db.query_one(&query).await?.unwrap();
    let value: u32 = row.try_get("", "number").unwrap();
    assert_eq!(value, 48u32);

    Ok(())
}

pub fn type_test() {
    assert_eq!(MyInteger::type_name(), "MyInteger");
    assert_eq!(StringVec::type_name(), "StringVec");

    assert_eq!(MyInteger::column_type(), ColumnType::Integer);
    assert_eq!(MyInteger::array_type(), ArrayType::Int);

    assert!(matches!(Tag1::column_type(), ColumnType::String(_)));
    assert_eq!(Tag1::array_type(), ArrayType::String);

    assert_eq!(
        StringVec::column_type(),
        ColumnType::Array(Arc::new(ColumnType::String(StringLen::None)))
    );
    assert_eq!(StringVec::array_type(), ArrayType::String);
}

pub fn conversion_test() {
    let stringvec = StringVec(vec!["ab".to_string(), "cd".to_string()]);
    let string: Value = stringvec.into();
    assert_eq!(
        string,
        Value::Array(
            ArrayType::String,
            Some(Box::new(vec![
                "ab".to_string().into(),
                "cd".to_string().into()
            ]))
        )
    );

    let value_random_int = Value::Int(Some(523));
    let unwrap_int = MyInteger::unwrap(value_random_int.clone());
    let try_from_int =
        <MyInteger as ValueType>::try_from(value_random_int).expect("should be ok to convert");

    // tests for unwrap and try_from
    let direct_int: MyInteger = 523.into();
    assert_eq!(direct_int, unwrap_int);
    assert_eq!(direct_int, try_from_int);

    // test for error
    let try_from_string_vec = <StringVec as ValueType>::try_from(Value::Char(Some('a')))
        .expect_err("should not be ok to convert char to stringvec");
    assert_eq!(try_from_string_vec.to_string(), ValueTypeErr.to_string());
}
