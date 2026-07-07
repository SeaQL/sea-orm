#![allow(unused_imports, dead_code)]

//! Runtime coverage for `SelectFour::consolidate().all()` (star topology),
//! mirroring the existing `SelectThree` consolidate test: build the 4-way query,
//! execute it, and check the cross-product rows consolidate into per-child `Vec`s.

mod common;

use crate::common::TestContext;
use pretty_assertions::assert_eq;
use sea_orm::{DbErr, QueryOrder, Set, entity::prelude::*, entity::*};

mod center {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "sfm_center")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        #[sea_orm(has_many)]
        pub a: HasMany<super::leaf_a::Entity>,
        #[sea_orm(has_many)]
        pub b: HasMany<super::leaf_b::Entity>,
        #[sea_orm(has_many)]
        pub c: HasMany<super::leaf_c::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

macro_rules! leaf {
    ($module:ident, $table:literal) => {
        mod $module {
            use sea_orm::entity::prelude::*;

            #[sea_orm::model]
            #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
            #[sea_orm(table_name = $table)]
            pub struct Model {
                #[sea_orm(primary_key)]
                pub id: i32,
                pub center_id: i32,
                pub label: String,
                #[sea_orm(belongs_to, from = "center_id", to = "id")]
                pub center: HasOne<super::center::Entity>,
            }

            impl ActiveModelBehavior for ActiveModel {}
        }
    };
}

leaf!(leaf_a, "sfm_a");
leaf!(leaf_b, "sfm_b");
leaf!(leaf_c, "sfm_c");

#[sea_orm_macros::test]
async fn select_four_many_consolidate_star() -> Result<(), DbErr> {
    let ctx = TestContext::new("select_four_many_tests").await;
    let db = &ctx.db;

    db.get_schema_builder()
        .register(center::Entity)
        .register(leaf_a::Entity)
        .register(leaf_b::Entity)
        .register(leaf_c::Entity)
        .apply(db)
        .await?;

    center::ActiveModel {
        id: Set(1),
        name: Set("hub".to_owned()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // Two children in each of the three has_many relations.
    for id in [1, 2] {
        leaf_a::ActiveModel {
            id: Set(id),
            center_id: Set(1),
            label: Set(format!("a{id}")),
            ..Default::default()
        }
        .insert(db)
        .await?;
        leaf_b::ActiveModel {
            id: Set(id),
            center_id: Set(1),
            label: Set(format!("b{id}")),
            ..Default::default()
        }
        .insert(db)
        .await?;
        leaf_c::ActiveModel {
            id: Set(id),
            center_id: Set(1),
            label: Set(format!("c{id}")),
            ..Default::default()
        }
        .insert(db)
        .await?;
    }

    // Build the 4-way star (center -> a, b, c), then consolidate the
    // a*b*c = 8-row cross product back into one tuple per center.
    // Order by each child's id so the consolidated child Vecs are deterministic
    // (mirrors the SelectThree consolidate test).
    let rows = center::Entity::find()
        .find_also_related(leaf_a::Entity)
        .find_also_related(leaf_b::Entity)
        .find_also(center::Entity, leaf_c::Entity)
        .order_by_asc(leaf_a::Column::Id)
        .order_by_asc(leaf_b::Column::Id)
        .order_by_asc(leaf_c::Column::Id)
        .consolidate()
        .all(db)
        .await?;

    assert_eq!(rows.len(), 1);
    let (hub, a, b, c) = &rows[0];
    assert_eq!(hub.id, 1);
    assert_eq!(
        a.iter().map(|m| m.label.as_str()).collect::<Vec<_>>(),
        ["a1", "a2"]
    );
    assert_eq!(
        b.iter().map(|m| m.label.as_str()).collect::<Vec<_>>(),
        ["b1", "b2"]
    );
    assert_eq!(
        c.iter().map(|m| m.label.as_str()).collect::<Vec<_>>(),
        ["c1", "c2"]
    );

    Ok(())
}
