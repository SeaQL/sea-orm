#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{TestContext, features::*, setup::*};
use pretty_assertions::assert_eq;
use sea_orm::{DatabaseConnection, DbBackend, ExprTrait, entity::prelude::*, entity::*};
use serde_json::json;

mod json_compact {
    use sea_orm::entity::prelude::*;

    #[sea_orm::compact_model]
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "json_compact")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(column_type = "JsonBinary")]
        pub json: Json,
        #[sea_orm(column_type = "JsonBinary", nullable)]
        pub json_opt: Option<Json>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

#[sea_orm_macros::test]
fn json_struct_tests() -> Result<(), DbErr> {
    let ctx = TestContext::new("json_struct_tests");
    create_tables(&ctx.db)?;
    insert_json_struct_1(&ctx.db)?;
    insert_json_struct_2(&ctx.db)?;
    insert_json_struct_3(&ctx.db)?;

    ctx.delete();

    Ok(())
}

#[sea_orm_macros::test]
#[should_panic(
    expected = "Failed to serialize 'NonSerializableStruct': Error(\"intentionally failing serialization\", line: 0, column: 0)"
)]
fn panic_on_non_serializable_insert() {
    use json_struct::*;

    let ctx = TestContext::new("json_struct_non_serializable_test");

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

    let _ = model.into_active_model().insert(&ctx.db);
}

pub fn insert_json_struct_1(db: &DatabaseConnection) -> Result<(), DbErr> {
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

    let result = model.clone().into_active_model().insert(db)?;

    assert_eq!(result, model);

    assert_eq!(
        Entity::find().filter(Column::Id.eq(model.id)).one(db)?,
        Some(model)
    );

    Ok(())
}

pub fn insert_json_struct_2(db: &DatabaseConnection) -> Result<(), DbErr> {
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

    let result = model.clone().into_active_model().insert(db)?;

    assert_eq!(result, model);

    assert_eq!(
        Entity::find().filter(Column::Id.eq(model.id)).one(db)?,
        Some(model)
    );

    Ok(())
}

pub fn insert_json_struct_3(db: &DatabaseConnection) -> Result<(), DbErr> {
    db.get_schema_builder()
        .register(json_compact::Entity)
        .apply(db)?;

    use json_compact::*;

    let model = Model {
        id: 3,
        json: json!({ "id": 22 }),
        json_opt: None,
    };

    let model_2 = model.into_active_model().insert(db)?;

    if db.get_database_backend() == DbBackend::MySql {
        // FIXME how can we abstract this?
        // MariaDb doesn't require CAST AS json
        if false {
            assert_eq!(
                Entity::find()
                    .filter(
                        Expr::col(COLUMN.json).eq(Expr::val(json!({ "id": 22 })).cast_as("json"))
                    )
                    .one(db)?
                    .unwrap(),
                model_2
            );
        }
    } else {
        assert_eq!(
            Entity::find()
                .filter(COLUMN.json.eq(json!({ "id": 22 })))
                .one(db)?
                .unwrap(),
            model_2
        );
    }

    let model = Model {
        id: 4,
        json: json!({ "id": 11 }),
        json_opt: Some(json!({ "id": 33 })),
    };

    let model_4 = model.into_active_model().insert(db)?;

    if db.get_database_backend() == DbBackend::MySql {
        // FIXME how can we abstract this?
        // MariaDb doesn't require CAST AS json
        if false {
            assert_eq!(
                Entity::find()
                    .filter(
                        Expr::col(COLUMN.json_opt)
                            .eq(Expr::val(json!({ "id": 33 })).cast_as("json"))
                    )
                    .one(db)?
                    .unwrap(),
                model_4
            );
        }
    } else {
        assert_eq!(
            Entity::find()
                .filter(COLUMN.json_opt.eq(json!({ "id": 33 })))
                .one(db)?
                .unwrap(),
            model_4
        );
    }

    assert_eq!(
        Entity::find()
            .filter(COLUMN.json_opt.eq(Option::<Json>::None))
            .one(db)?
            .unwrap(),
        model_2
    );

    Ok(())
}
