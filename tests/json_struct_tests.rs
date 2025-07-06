#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{TestContext, features::*, setup::*};
use pretty_assertions::assert_eq;
use sea_orm::{DatabaseConnection, entity::prelude::*, entity::*};
use serde_json::json;

#[sea_orm_macros::test]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("json_struct_tests").await;
    create_tables(&ctx.db).await?;
    insert_json_struct_1(&ctx.db).await?;
    insert_json_struct_2(&ctx.db).await?;

    ctx.delete().await;

    Ok(())
}

#[sea_orm_macros::test]
#[should_panic(
    expected = "Failed to serialize 'NonSerializableStruct': Error(\"intentionally failing serialization\", line: 0, column: 0)"
)]
async fn panic_on_non_serializable_insert() {
    use json_struct::*;

    let ctx = TestContext::new("json_struct_non_serializable_test").await;

    let model = Model {
        id: 1,
        json: json!({
            "id": 1,
            "name": "apple",
            "price": 12.01,
            "notes": "hand picked, organic",
        }),
        json_value: KeyValue {
            id: 1,
            name: "apple".into(),
            price: 12.01,
            notes: Some("hand picked, organic".into()),
        },
        json_value_opt: Some(KeyValue {
            id: 1,
            name: "apple".into(),
            price: 12.01,
            notes: Some("hand picked, organic".into()),
        }),
        json_non_serializable: Some(NonSerializableStruct),
    };

    let _ = model.into_active_model().insert(&ctx.db).await;
}

pub async fn insert_json_struct_1(db: &DatabaseConnection) -> Result<(), DbErr> {
    use json_struct::*;

    let model = Model {
        id: 1,
        json: json!({
            "id": 1,
            "name": "apple",
            "price": 12.01,
            "notes": "hand picked, organic",
        }),
        json_value: KeyValue {
            id: 1,
            name: "apple".into(),
            price: 12.01,
            notes: Some("hand picked, organic".into()),
        },
        json_value_opt: Some(KeyValue {
            id: 1,
            name: "apple".into(),
            price: 12.01,
            notes: Some("hand picked, organic".into()),
        }),
        json_non_serializable: None,
    };

    let result = model.clone().into_active_model().insert(db).await?;

    assert_eq!(result, model);

    assert_eq!(
        Entity::find()
            .filter(Column::Id.eq(model.id))
            .one(db)
            .await?,
        Some(model)
    );

    Ok(())
}

pub async fn insert_json_struct_2(db: &DatabaseConnection) -> Result<(), DbErr> {
    use json_struct::*;

    let model = Model {
        id: 2,
        json: json!({
            "id": 2,
            "name": "orange",
            "price": 10.93,
            "notes": "sweet & juicy",
        }),
        json_value: KeyValue {
            id: 1,
            name: "orange".into(),
            price: 10.93,
            notes: None,
        },
        json_value_opt: None,
        json_non_serializable: None,
    };

    let result = model.clone().into_active_model().insert(db).await?;

    assert_eq!(result, model);

    assert_eq!(
        Entity::find()
            .filter(Column::Id.eq(model.id))
            .one(db)
            .await?,
        Some(model)
    );

    Ok(())
}
