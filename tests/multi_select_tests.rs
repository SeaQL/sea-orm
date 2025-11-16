#![allow(unused_imports, dead_code)]

mod common;

use crate::common::TestContext;
use sea_orm::{
    Database, DbConn, DbErr,
    entity::*,
    query::*,
    sea_query::{Expr, Query},
};

mod one {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "one")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(has_one)]
        pub two: Option<super::two::Entity>,
        #[sea_orm(has_one)]
        pub six: Option<super::six::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod two {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "two")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub one_id: i32,
        pub three_id: i32,
        #[sea_orm(belongs_to, from = "one_id", to = "id")]
        pub one: Option<super::one::Entity>,
        #[sea_orm(belongs_to, from = "three_id", to = "id")]
        pub three: Option<super::three::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod three {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "three")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(has_one)]
        pub four: Option<super::four::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod four {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "four")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub three_id: Option<i32>,
        #[sea_orm(belongs_to, from = "three_id", to = "id")]
        pub three: Option<super::three::Entity>,
        #[sea_orm(has_one)]
        pub five: Option<super::five::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod five {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "five")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub four_id: i32,
        #[sea_orm(belongs_to, from = "four_id", to = "id")]
        pub four: Option<super::four::Entity>,
        #[sea_orm(has_one)]
        pub six: Option<super::six::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod six {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "six")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub one_id: i32,
        pub five_id: i32,
        #[sea_orm(belongs_to, from = "one_id", to = "id")]
        pub one: Option<super::one::Entity>,
        #[sea_orm(belongs_to, from = "five_id", to = "id")]
        pub five: Option<super::five::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod composite_a {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "composite_a")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(unique_key = "pair")]
        pub left_id: i32,
        #[sea_orm(unique_key = "pair")]
        pub right_id: i32,
        #[sea_orm(has_one)]
        pub b: Option<super::composite_b::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod composite_b {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "composite_b")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub left_id: i32,
        pub right_id: i32,
        #[sea_orm(belongs_to, from = "(left_id, right_id)", to = "(left_id, right_id)")]
        pub a: Option<super::composite_a::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod composite_b_without_index {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "composite_b")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub left_id: i32,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod composite_c {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "composite_c")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(unique_key = "season_episode")]
        pub season_number: i32,
        #[sea_orm(unique_key = "season_episode")]
        pub episode_number: i32,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

#[sea_orm_macros::test]
async fn test_select_six() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_select_six").await;
    let db = &ctx.db;

    db.get_schema_builder()
        .register(one::Entity)
        .register(two::Entity)
        .register(three::Entity) // topologically three ranks higher than two
        .register(four::Entity)
        .register(five::Entity)
        .register(six::Entity)
        .apply(db)
        .await?;

    one::ActiveModel { id: Set(1) }.insert(db).await?;
    one::ActiveModel { id: Set(11) }.insert(db).await?;

    three::ActiveModel { id: Set(3) }.insert(db).await?;
    three::ActiveModel { id: Set(33) }.insert(db).await?;

    two::ActiveModel {
        id: Set(2),
        one_id: Set(1),
        three_id: Set(3),
    }
    .insert(db)
    .await?;

    four::ActiveModel {
        id: Set(4),
        three_id: Set(Some(3)),
    }
    .insert(db)
    .await?;

    four::ActiveModel {
        id: Set(44),
        three_id: Set(None),
    }
    .insert(db)
    .await?;

    five::ActiveModel {
        id: Set(5),
        four_id: Set(4),
    }
    .insert(db)
    .await?;

    five::ActiveModel {
        id: Set(55),
        four_id: Set(44),
    }
    .insert(db)
    .await?;

    six::ActiveModel {
        id: Set(6),
        one_id: Set(1),
        five_id: Set(55),
    }
    .insert(db)
    .await?;

    let one = one::Entity::find()
        .order_by_id_asc()
        .one(db)
        .await?
        .unwrap();
    assert_eq!(one.id, 1);

    let two = two::Entity::find()
        .order_by_id_asc()
        .one(db)
        .await?
        .unwrap();
    assert_eq!(two.id, 2);

    let three = three::Entity::find()
        .order_by_id_asc()
        .one(db)
        .await?
        .unwrap();
    assert_eq!(three.id, 3);

    let four = four::Entity::find()
        .order_by_id_asc()
        .one(db)
        .await?
        .unwrap();
    assert_eq!(four.id, 4);

    let five = five::Entity::find()
        .order_by_id_asc()
        .one(db)
        .await?
        .unwrap();
    assert_eq!(five.id, 5);

    let six = six::Entity::find()
        .order_by_id_asc()
        .one(db)
        .await?
        .unwrap();
    assert_eq!(six.id, 6);

    let (one, two) = one::Entity::find()
        .find_also(one::Entity, two::Entity)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(one.id, 1);
    assert_eq!(two.unwrap().id, 2);

    let (one, two, three) = one::Entity::find()
        .find_also(one::Entity, two::Entity)
        .find_also(two::Entity, three::Entity)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(one.id, 1);
    assert_eq!(two.unwrap().id, 2);
    assert_eq!(three.unwrap().id, 3);

    let (two, one, three) = two::Entity::find()
        .find_also_related(one::Entity)
        .find_also_related(three::Entity)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(one.unwrap().id, 1);
    assert_eq!(two.id, 2);
    assert_eq!(three.unwrap().id, 3);

    let (one, two, three, four) = one::Entity::find()
        .find_also(one::Entity, two::Entity)
        .find_also(two::Entity, three::Entity)
        .find_also(three::Entity, four::Entity)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(one.id, 1);
    assert_eq!(two.unwrap().id, 2);
    assert_eq!(three.unwrap().id, 3);
    assert_eq!(four.unwrap().id, 4);

    let (one, two, three, six) = one::Entity::find()
        .find_also(one::Entity, two::Entity)
        .find_also(two::Entity, three::Entity)
        .find_also(one::Entity, six::Entity)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(one.id, 1);
    assert_eq!(two.unwrap().id, 2);
    assert_eq!(three.unwrap().id, 3);
    assert_eq!(six.unwrap().id, 6);

    let (one, two, three, four, five) = one::Entity::find()
        .find_also(one::Entity, two::Entity)
        .find_also(two::Entity, three::Entity)
        .find_also(three::Entity, four::Entity)
        .find_also(four::Entity, five::Entity)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(one.id, 1);
    assert_eq!(two.unwrap().id, 2);
    assert_eq!(three.unwrap().id, 3);
    assert_eq!(four.unwrap().id, 4);
    assert_eq!(five.unwrap().id, 5);

    let (one, two, three, four, five, six) = one::Entity::find()
        .find_also(one::Entity, two::Entity)
        .find_also(two::Entity, three::Entity)
        .find_also(three::Entity, four::Entity)
        .find_also(four::Entity, five::Entity)
        .find_also(one::Entity, six::Entity)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(one.id, 1);
    assert_eq!(two.unwrap().id, 2);
    assert_eq!(three.unwrap().id, 3);
    assert_eq!(four.unwrap().id, 4);
    assert_eq!(five.unwrap().id, 5);
    assert_eq!(six.unwrap().id, 6);

    let (six, five, four) = six::Entity::find_by_id(6)
        .find_also_related(five::Entity)
        .and_also_related(four::Entity)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(four.unwrap().id, 44);
    assert_eq!(five.unwrap().id, 55);
    assert_eq!(six.id, 6);

    let (four, five, six) = four::Entity::find_by_id(44)
        .find_also_related(five::Entity)
        .and_also_related(six::Entity)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(four.id, 44);
    assert_eq!(five.unwrap().id, 55);
    assert_eq!(six.unwrap().id, 6);

    // below is EntityLoader

    let one_ex = one::Entity::load().one(db).await?.unwrap();
    assert_eq!(one_ex.id, 1);

    let one_ex = one::Entity::load()
        .with(two::Entity)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(one_ex.id, 1);
    assert_eq!(one_ex.two.unwrap().id, 2);

    let one_ex = one::Entity::load()
        .with((two::Entity, three::Entity))
        .one(db)
        .await?
        .unwrap();
    assert_eq!(one_ex.id, 1);
    let two = one_ex.two.unwrap();
    assert_eq!(two.id, 2);
    assert_eq!(two.three.unwrap().id, 3);

    let one_ex = one::Entity::load()
        .with(six::Entity)
        .with((two::Entity, three::Entity))
        .one(db)
        .await?
        .unwrap();
    assert_eq!(one_ex.id, 1);
    let two = one_ex.two.unwrap();
    assert_eq!(two.id, 2);
    assert_eq!(two.three.unwrap().id, 3);
    let six = one_ex.six.unwrap();
    assert_eq!(six.id, 6);

    let one_ex = one::Entity::load()
        .with((six::Entity, five::Entity))
        .with((two::Entity, three::Entity))
        .one(db)
        .await?
        .unwrap();
    assert_eq!(one_ex.id, 1);
    let two = one_ex.two.unwrap();
    assert_eq!(two.id, 2);
    assert_eq!(two.three.unwrap().id, 3);
    let six = one_ex.six.unwrap();
    assert_eq!(six.id, 6);
    assert_eq!(six.five.unwrap().id, 55);

    Ok(())
}

#[sea_orm_macros::test]
async fn test_composite_foreign_key() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_composite_foreign_key").await;
    let db = &ctx.db;

    #[cfg(not(feature = "schema-sync"))]
    {
        db.get_schema_builder()
            .register(composite_b::Entity)
            .register(composite_a::Entity) // should create a first
            .apply(db)
            .await?;
    }

    #[cfg(feature = "schema-sync")]
    {
        // create 1 table only
        db.get_schema_builder()
            .register(composite_a::Entity)
            .apply(db)
            .await?;

        // this incrementally creates the 2nd table without index
        if db.get_database_backend() != sea_orm::DbBackend::Sqlite {
            db.get_schema_builder()
                .register(composite_a::Entity)
                .register(composite_b_without_index::Entity)
                .sync(db)
                .await?;
        }

        // this creates the full 2nd table
        db.get_schema_builder()
            .register(composite_a::Entity)
            .register(composite_b::Entity)
            .register(composite_c::Entity)
            .sync(db)
            .await?;

        // sync again just to be sure
        db.get_schema_builder()
            .register(composite_a::Entity)
            .register(composite_b::Entity)
            .register(composite_c::Entity)
            .sync(db)
            .await?;

        #[cfg(feature = "sqlx-postgres")]
        {
            let has_index: bool = db
                .query_one(
                    Query::select()
                        .expr(Expr::cust("COUNT(*) > 0"))
                        .from("pg_indexes")
                        .cond_where(
                            Condition::all()
                                .add(Expr::cust("schemaname = CURRENT_SCHEMA()"))
                                .add(Expr::col("tablename").eq("composite_c"))
                                .add(Expr::col("indexname").eq("idx-composite_c-season_episode")),
                        ),
                )
                .await?
                .unwrap()
                .try_get_by_index(0)
                .unwrap();

            assert!(has_index, "Should have index on `composite_c`");
        }
    }

    composite_a::ActiveModel {
        id: Set(100),
        left_id: Set(1),
        right_id: Set(2),
    }
    .insert(db)
    .await?;
    composite_a::ActiveModel {
        id: Set(101),
        left_id: Set(1),
        right_id: Set(3),
    }
    .insert(db)
    .await?;
    composite_a::ActiveModel {
        id: Set(102),
        left_id: Set(2),
        right_id: Set(3),
    }
    .insert(db)
    .await?;
    composite_b::ActiveModel {
        id: Set(200),
        left_id: Set(1),
        right_id: Set(2),
    }
    .insert(db)
    .await?;
    composite_b::ActiveModel {
        id: Set(202),
        left_id: Set(2),
        right_id: Set(3),
    }
    .insert(db)
    .await?;

    let a = composite_a::Entity::find_by_id(100).one(db).await?.unwrap();
    assert_eq!(a.left_id, 1);
    assert_eq!(a.right_id, 2);
    #[rustfmt::skip]
    assert_eq!(composite_a::Entity::find_by_pair((1, 3)).one(db).await?.unwrap().id, 101);
    #[rustfmt::skip]
    assert_eq!(composite_a::Entity::load().filter_by_pair((1, 3)).one(db).await?.unwrap().id, 101);

    let Some((a, Some(b))) = composite_a::Entity::find_by_id(100)
        .find_also_related(composite_b::Entity)
        .one(db)
        .await?
    else {
        panic!("query error")
    };
    assert_eq!(a.left_id, 1);
    assert_eq!(a.right_id, 2);
    assert_eq!(b.id, 200);
    assert_eq!(b.left_id, 1);
    assert_eq!(b.right_id, 2);

    let a = composite_a::Entity::load()
        .filter_by_id(101)
        .with(composite_b::Entity)
        .one(db)
        .await?
        .unwrap();

    assert_eq!(a.id, 101);
    assert!(a.b.is_not_found());

    let a = composite_a::Entity::load()
        .filter_by_id(102)
        .with(composite_b::Entity)
        .one(db)
        .await?
        .unwrap();

    assert_eq!(a.id, 102);
    assert_eq!(a.left_id, 2);
    assert_eq!(a.right_id, 3);
    assert_eq!(a.b.unwrap().id, 202);

    assert_eq!(
        composite_b::Entity::delete_by_id(202)
            .exec(db)
            .await?
            .rows_affected,
        1
    );

    assert_eq!(
        composite_a::Entity::delete_by_pair((2, 3))
            .exec(db)
            .await?
            .rows_affected,
        1
    );

    assert!(
        composite_a::Entity::find_by_id(102)
            .one(db)
            .await?
            .is_none()
    );

    Ok(())
}
