pub mod common;

pub use common::{features::*, setup::*, TestContext};
use pretty_assertions::assert_eq;
use sea_orm::{entity::prelude::*, DerivePartialModel, FromQueryResult, Set};
use serde_json::json;

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("cursor_tests").await;
    create_tables(&ctx.db).await?;
    create_insert_default(&ctx.db).await?;
    cursor_pagination(&ctx.db).await?;
    schema::create_tables(&ctx.db).await?;
    create_baker_cake(&ctx.db).await?;
    cursor_related_pagination(&ctx.db).await?;
    ctx.delete().await;

    Ok(())
}

pub async fn create_insert_default(db: &DatabaseConnection) -> Result<(), DbErr> {
    use insert_default::*;

    for _ in 0..10 {
        ActiveModel {
            ..Default::default()
        }
        .insert(db)
        .await?;
    }

    assert_eq!(
        Entity::find().all(db).await?,
        [
            Model { id: 1 },
            Model { id: 2 },
            Model { id: 3 },
            Model { id: 4 },
            Model { id: 5 },
            Model { id: 6 },
            Model { id: 7 },
            Model { id: 8 },
            Model { id: 9 },
            Model { id: 10 },
        ]
    );

    Ok(())
}

pub async fn cursor_pagination(db: &DatabaseConnection) -> Result<(), DbErr> {
    use insert_default::*;

    // Before 5, i.e. id < 5

    let mut cursor = Entity::find().cursor_by(Column::Id);

    cursor.before(5);

    assert_eq!(
        cursor.first(4).all(db).await?,
        [
            Model { id: 1 },
            Model { id: 2 },
            Model { id: 3 },
            Model { id: 4 },
        ]
    );

    assert_eq!(
        cursor.first(5).all(db).await?,
        [
            Model { id: 1 },
            Model { id: 2 },
            Model { id: 3 },
            Model { id: 4 },
        ]
    );

    assert_eq!(
        cursor.last(4).all(db).await?,
        [
            Model { id: 1 },
            Model { id: 2 },
            Model { id: 3 },
            Model { id: 4 },
        ]
    );

    assert_eq!(
        cursor.last(5).all(db).await?,
        [
            Model { id: 1 },
            Model { id: 2 },
            Model { id: 3 },
            Model { id: 4 },
        ]
    );

    // After 5, i.e. id > 5

    let mut cursor = Entity::find().cursor_by(Column::Id);

    cursor.after(5);

    assert_eq!(
        cursor.first(4).all(db).await?,
        [
            Model { id: 6 },
            Model { id: 7 },
            Model { id: 8 },
            Model { id: 9 },
        ]
    );

    assert_eq!(
        cursor.first(5).all(db).await?,
        [
            Model { id: 6 },
            Model { id: 7 },
            Model { id: 8 },
            Model { id: 9 },
            Model { id: 10 },
        ]
    );

    assert_eq!(
        cursor.first(6).all(db).await?,
        [
            Model { id: 6 },
            Model { id: 7 },
            Model { id: 8 },
            Model { id: 9 },
            Model { id: 10 },
        ]
    );

    assert_eq!(
        cursor.last(4).all(db).await?,
        [
            Model { id: 7 },
            Model { id: 8 },
            Model { id: 9 },
            Model { id: 10 },
        ]
    );

    assert_eq!(
        cursor.last(5).all(db).await?,
        [
            Model { id: 6 },
            Model { id: 7 },
            Model { id: 8 },
            Model { id: 9 },
            Model { id: 10 },
        ]
    );

    assert_eq!(
        cursor.last(6).all(db).await?,
        [
            Model { id: 6 },
            Model { id: 7 },
            Model { id: 8 },
            Model { id: 9 },
            Model { id: 10 },
        ]
    );

    // Between 5 and 8, i.e. id > 5 AND id < 8

    let mut cursor = Entity::find().cursor_by(Column::Id);

    cursor.after(5).before(8);

    assert_eq!(cursor.first(1).all(db).await?, [Model { id: 6 }]);

    assert_eq!(
        cursor.first(2).all(db).await?,
        [Model { id: 6 }, Model { id: 7 }]
    );

    assert_eq!(
        cursor.first(3).all(db).await?,
        [Model { id: 6 }, Model { id: 7 }]
    );

    assert_eq!(cursor.last(1).all(db).await?, [Model { id: 7 }]);

    assert_eq!(
        cursor.last(2).all(db).await?,
        [Model { id: 6 }, Model { id: 7 }]
    );

    assert_eq!(
        cursor.last(3).all(db).await?,
        [Model { id: 6 }, Model { id: 7 }]
    );

    // Fetch custom struct

    #[derive(FromQueryResult, Debug, PartialEq, Clone)]
    struct Row {
        id: i32,
    }

    let mut cursor = cursor.into_model::<Row>();

    assert_eq!(
        cursor.first(2).all(db).await?,
        [Row { id: 6 }, Row { id: 7 }]
    );

    assert_eq!(
        cursor.first(3).all(db).await?,
        [Row { id: 6 }, Row { id: 7 }]
    );

    // Fetch JSON value

    let mut cursor = cursor.into_json();

    assert_eq!(
        cursor.first(2).all(db).await?,
        [json!({ "id": 6 }), json!({ "id": 7 })]
    );

    assert_eq!(
        cursor.first(3).all(db).await?,
        [json!({ "id": 6 }), json!({ "id": 7 })]
    );

    #[derive(DerivePartialModel, FromQueryResult, Debug, PartialEq, Clone)]
    #[sea_orm(entity = "Entity")]
    struct PartialRow {
        #[sea_orm(from_col = "id")]
        id: i32,
        #[sea_orm(from_expr = "sea_query::Expr::col(Column::Id).add(1000)")]
        id_shifted: i32,
    }

    let mut cursor = cursor.into_partial_model::<PartialRow>();

    assert_eq!(
        cursor.first(2).all(db).await?,
        [
            PartialRow {
                id: 6,
                id_shifted: 1006,
            },
            PartialRow {
                id: 7,
                id_shifted: 1007,
            }
        ]
    );

    assert_eq!(
        cursor.first(3).all(db).await?,
        [
            PartialRow {
                id: 6,
                id_shifted: 1006,
            },
            PartialRow {
                id: 7,
                id_shifted: 1007,
            }
        ]
    );

    Ok(())
}

use common::bakery_chain::{
    baker, bakery, cake, cakes_bakers, schema, Baker, Bakery, Cake, CakesBakers,
};

fn bakery(i: i32) -> bakery::Model {
    bakery::Model {
        name: i.to_string(),
        profit_margin: 10.4,
        id: i,
    }
}
fn baker(c: char) -> baker::Model {
    baker::Model {
        name: c.clone().to_string(),
        contact_details: serde_json::json!({
            "mobile": "+61424000000",
        }),
        bakery_id: Some((c as i32 - 65) % 10 + 1),
        id: c as i32 - 64,
    }
}

fn cake(c: char) -> cake::Model {
    cake::Model {
        name: c.to_ascii_lowercase().to_string(),
        price: rust_decimal_macros::dec!(10.25),
        gluten_free: false,
        serial: Uuid::new_v4(),
        bakery_id: Some((c as i32 - 65) % 10 + 1),
        id: c as i32 - 64,
    }
}

pub async fn create_baker_cake(db: &DatabaseConnection) -> Result<(), DbErr> {
    use sea_orm::IntoActiveModel;

    let mut bakeries: Vec<bakery::ActiveModel> = vec![];
    // bakeries named from 1 to 10
    for i in 1..=10 {
        bakeries.push(bakery::ActiveModel {
            name: Set(i.to_string()),
            profit_margin: Set(10.4),
            ..Default::default()
        });
    }
    let _ = Bakery::insert_many(bakeries).exec(db).await?;

    let mut bakers: Vec<baker::ActiveModel> = vec![];
    let mut cakes: Vec<cake::ActiveModel> = vec![];
    let mut cakes_bakers: Vec<cakes_bakers::ActiveModel> = vec![];
    // baker and cakes named from "A" to "Z" and from "a" to "z"
    for c in 'A'..='Z' {
        bakers.push(baker::ActiveModel {
            name: Set(c.clone().to_string()),
            contact_details: Set(serde_json::json!({
                "mobile": "+61424000000",
            })),
            bakery_id: Set(Some((c as i32 - 65) % 10 + 1)),
            ..Default::default()
        });
        cakes.push(cake::ActiveModel {
            name: Set(c.to_ascii_lowercase().to_string()),
            price: Set(rust_decimal_macros::dec!(10.25)),
            gluten_free: Set(false),
            serial: Set(Uuid::new_v4()),
            bakery_id: Set(Some((c as i32 - 65) % 10 + 1)),
            ..Default::default()
        });
        cakes_bakers.push(cakes_bakers::ActiveModel {
            cake_id: Set(c as i32 - 64),
            baker_id: Set(c as i32 - 64),
        })
    }
    Baker::insert_many(bakers).exec(db).await?;
    Cake::insert_many(cakes).exec(db).await?;
    CakesBakers::insert_many(cakes_bakers).exec(db).await?;

    Ok(())
}

pub async fn cursor_related_pagination(db: &DatabaseConnection) -> Result<(), DbErr> {
    use common::bakery_chain::*;

    assert_eq!(
        bakery::Entity::find()
            .cursor_by(bakery::Column::Id)
            .before(5)
            .first(4)
            .all(db)
            .await?,
        [bakery(1), bakery(2), bakery(3), bakery(4),]
    );

    assert_eq!(
        bakery::Entity::find()
            .find_also_related(Baker)
            .cursor_by(bakery::Column::Id)
            .before(5)
            .first(20)
            .all(db)
            .await?,
        [
            (bakery(1), Some(baker('A'))),
            (bakery(1), Some(baker('K'))),
            (bakery(1), Some(baker('U'))),
            (bakery(2), Some(baker('B'))),
            (bakery(2), Some(baker('L'))),
            (bakery(2), Some(baker('V'))),
            (bakery(3), Some(baker('C'))),
            (bakery(3), Some(baker('M'))),
            (bakery(3), Some(baker('W'))),
            (bakery(4), Some(baker('D'))),
            (bakery(4), Some(baker('N'))),
            (bakery(4), Some(baker('X'))),
        ]
    );

    assert_eq!(
        bakery::Entity::find()
            .find_also_related(Baker)
            .cursor_by(bakery::Column::Id)
            .before(5)
            .first(4)
            .all(db)
            .await?,
        [
            (bakery(1), Some(baker('A'))),
            (bakery(1), Some(baker('K'))),
            (bakery(1), Some(baker('U'))),
            (bakery(2), Some(baker('B'))),
        ]
    );

    // since 10 is before 2 lexicologically, it return that first
    assert_eq!(
        bakery::Entity::find()
            .find_also_related(Baker)
            .cursor_by(bakery::Column::Name)
            .before("3")
            .first(4)
            .all(db)
            .await?,
        [
            (bakery(1), Some(baker('A'))),
            (bakery(1), Some(baker('K'))),
            (bakery(1), Some(baker('U'))),
            (bakery(10), Some(baker('J'))),
        ]
    );

    Ok(())
}
