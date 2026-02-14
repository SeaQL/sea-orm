#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{TestContext, features::*, setup::*};
use pretty_assertions::assert_eq;
use sea_orm::{DatabaseConnection, entity::prelude::*, entity::*};
use serde_json::json;

#[sea_orm_macros::test]
fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("bakery_chain_uuid_tests");
    create_metadata_table(&ctx.db)?;
    create_and_update_metadata(&ctx.db)?;
    insert_metadata(&ctx.db)?;
    ctx.delete();

    Ok(())
}

pub fn insert_metadata(db: &DatabaseConnection) -> Result<(), DbErr> {
    let metadata = metadata::Model {
        uuid: Uuid::new_v4(),
        ty: "Type".to_owned(),
        key: "markup".to_owned(),
        value: "1.18".to_owned(),
        bytes: vec![1, 2, 3],
        date: Some(Date::from_ymd_opt(2021, 9, 27).unwrap()),
        time: Some(Time::from_hms_opt(11, 32, 55).unwrap()),
    };

    let result = metadata.clone().into_active_model().insert(db)?;

    assert_eq!(result, metadata);

    let mut json = metadata::Entity::find()
        .filter(metadata::Column::Uuid.eq(metadata.uuid))
        .into_json()
        .one(db)?;

    #[cfg(feature = "rusqlite")]
    {
        json.as_mut()
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove("uuid");
        // rusqlite current has no rich type info to properly deserialize a uuid
        assert_eq!(
            json,
            Some(json!({
                "type": metadata.ty,
                "key": metadata.key,
                "value": metadata.value,
                "bytes": metadata.bytes,
                "date": metadata.date,
                "time": metadata.time,
            }))
        );
    }
    #[cfg(not(feature = "rusqlite"))]
    {
        assert_eq!(
            json,
            Some(json!({
                "uuid": metadata.uuid,
                "type": metadata.ty,
                "key": metadata.key,
                "value": metadata.value,
                "bytes": metadata.bytes,
                "date": metadata.date,
                "time": metadata.time,
            }))
        );
    }

    Ok(())
}

pub fn create_and_update_metadata(db: &DatabaseConnection) -> Result<(), DbErr> {
    let metadata = metadata::Model {
        uuid: Uuid::new_v4(),
        ty: "Type".to_owned(),
        key: "markup".to_owned(),
        value: "1.18".to_owned(),
        bytes: vec![1, 2, 3],
        date: Some(Date::from_ymd_opt(2021, 9, 27).unwrap()),
        time: Some(Time::from_hms_opt(11, 32, 55).unwrap()),
    };

    let res = Metadata::insert(metadata.clone().into_active_model()).exec(db)?;

    assert_eq!(Metadata::find().one(db)?, Some(metadata.clone()));

    assert_eq!(res.last_insert_id, metadata.uuid);

    let update_res = Metadata::update(metadata::ActiveModel {
        value: Set("0.22".to_owned()),
        ..metadata.clone().into_active_model()
    })
    .validate()?
    .filter(metadata::Column::Uuid.eq(Uuid::default()))
    .exec(db);

    assert_eq!(update_res, Err(DbErr::RecordNotUpdated));

    Ok(())
}
