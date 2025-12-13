#![allow(unused_imports, dead_code)]

pub mod common;

use std::sync::Arc;
use std::vec;

pub use common::{
    TestContext,
    features::{
        value_type::{
            MyInteger, StringVec, Tag1, Tag2, Tag3, Tag4, Tag5, value_type_general, value_type_pg,
            value_type_pk,
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
    let str_vec = vec!["ab", "cd"]
        .into_iter()
        .map(|x| x.to_string())
        .map(Some)
        .collect();
    let model = value_type_pg::Model {
        id: 1,
        number: 48.into(),
        str_vec: StringVec(str_vec),
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

    assert!(matches!(Tag3::column_type(), ColumnType::String(_)));
    assert_eq!(Tag3::array_type(), ArrayType::String);

    assert!(matches!(Tag4::column_type(), ColumnType::String(_)));
    assert_eq!(Tag4::array_type(), ArrayType::String);

    assert!(matches!(Tag5::column_type(), ColumnType::String(_)));
    assert_eq!(Tag5::array_type(), ArrayType::String);

    let tag_3 = Tag3 { i: 22 };
    let tag_3_val: Value = tag_3.into();
    assert_eq!(tag_3_val, "22".into());
    let tag_3_: Tag3 = ValueType::try_from(tag_3_val).unwrap();
    assert_eq!(tag_3_, tag_3);

    let tag_4 = Tag4(22);
    let tag_4_val: Value = tag_4.into();
    assert_eq!(tag_4_val, "22".into());
    let tag_4_: Tag4 = ValueType::try_from(tag_4_val).unwrap();
    assert_eq!(tag_4_, tag_4);

    let tag_5 = Tag5(std::path::PathBuf::from("foo/bar"));
    let tag_5_val: Value = tag_5.clone().into();
    assert_eq!(tag_5_val, "foo/bar".into());
    let tag_5_: Tag5 = ValueType::try_from(tag_5_val).unwrap();
    assert_eq!(tag_5_, tag_5);

    assert_eq!(
        StringVec::column_type(),
        ColumnType::Array(Arc::new(ColumnType::String(StringLen::None)))
    );
    assert_eq!(StringVec::array_type(), ArrayType::String);
}

pub fn conversion_test() {
    let vec: Vec<_> = vec!["ab", "cd"]
        .into_iter()
        .map(|x| x.to_string())
        .map(Some)
        .collect();
    let stringvec = StringVec(vec.clone());
    let string: Value = stringvec.into();
    let array = sea_query::Array::String(vec.into());
    assert_eq!(string, Value::Array(Some(array)));

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
