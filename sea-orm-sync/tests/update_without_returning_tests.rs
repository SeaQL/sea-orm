#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{TestContext, features::*, setup::*};
use pretty_assertions::assert_eq;
use sea_orm::{DatabaseConnection, entity::prelude::*, entity::*};
use serde_json::json;

#[sea_orm_macros::test]
fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("update_without_returning_tests");
    create_repository_table(&ctx.db)?;
    create_edit_log_table(&ctx.db)?;
    update_without_returning(&ctx.db)?;
    update_without_returning_record_not_updated(&ctx.db)?;
    ctx.delete();

    Ok(())
}

// `update_without_returning` should update the row, run `before_save`, and
// intentionally skip `after_save`.
pub fn update_without_returning(db: &DatabaseConnection) -> Result<(), DbErr> {
    let model = repository::Model {
        id: "uwr-001".to_owned(),
        owner: "GC".to_owned(),
        name: "G.C.".to_owned(),
        description: None,
    };

    // Instance insert runs `before_save` + `after_save` (edit_log id 1 and 2).
    model.clone().into_active_model().insert(db)?;

    let updated = repository::ActiveModel {
        description: Set(Some("updated".to_owned())),
        ..model.clone().into_active_model()
    };

    let res = updated.update_without_returning(db)?;
    assert_eq!(res.rows_affected, 1);

    // The row is actually updated.
    assert_eq!(
        Repository::find_by_id("uwr-001".to_owned()).one(db)?,
        Some(repository::Model {
            description: Some("updated".to_owned()),
            ..model
        })
    );

    // `before_save` ran for the update (id 3), but `after_save` did NOT.
    assert_eq!(
        edit_log::Entity::find().all(db)?,
        [
            edit_log::Model {
                id: 1,
                action: "before_save".into(),
                values: json!({
                    "description": null,
                    "id": "uwr-001",
                    "name": "G.C.",
                    "owner": "GC",
                }),
            },
            edit_log::Model {
                id: 2,
                action: "after_save".into(),
                values: json!({
                    "description": null,
                    "id": "uwr-001",
                    "name": "G.C.",
                    "owner": "GC",
                }),
            },
            edit_log::Model {
                id: 3,
                action: "before_save".into(),
                values: json!({
                    "description": "updated",
                    "id": "uwr-001",
                    "name": "G.C.",
                    "owner": "GC",
                }),
            },
        ]
    );

    Ok(())
}

// Updating a row that does not exist returns `RecordNotUpdated`.
pub fn update_without_returning_record_not_updated(db: &DatabaseConnection) -> Result<(), DbErr> {
    let missing = repository::ActiveModel {
        id: Set("does-not-exist".to_owned()),
        owner: Set("GC".to_owned()),
        name: Set("G.C.".to_owned()),
        description: Set(Some("nope".to_owned())),
    };

    let res = missing.update_without_returning(db);
    assert_eq!(res.map(|_| ()), Err(DbErr::RecordNotUpdated));

    Ok(())
}
