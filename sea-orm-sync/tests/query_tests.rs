#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{TestContext, bakery_chain::*, bakery_dense, setup::*};
pub use sea_orm::entity::*;
pub use sea_orm::{ConnectionTrait, QueryFilter, QuerySelect};

// Run the test locally:
// DATABASE_URL="mysql://root:@localhost" cargo test --features sqlx-mysql,runtime-async-std --test query_tests
#[sea_orm_macros::test]
pub fn find_one_with_no_result() {
    let ctx = TestContext::new("find_one_with_no_result");
    create_tables(&ctx.db).unwrap();

    let bakery = Bakery::find().one(&ctx.db).unwrap();
    assert_eq!(bakery, None);

    ctx.delete();
}

#[sea_orm_macros::test]
pub fn find_one_with_result() {
    let ctx = TestContext::new("find_one_with_result");
    create_tables(&ctx.db).unwrap();

    let bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&ctx.db)
    .expect("could not insert bakery");

    let result = Bakery::find().one(&ctx.db).unwrap().unwrap();

    assert_eq!(result.id, bakery.id.unwrap());

    ctx.delete();
}

#[sea_orm_macros::test]
pub fn find_by_id_with_no_result() {
    let ctx = TestContext::new("find_by_id_with_no_result");
    create_tables(&ctx.db).unwrap();

    let bakery = Bakery::find_by_id(999).one(&ctx.db).unwrap();
    assert_eq!(bakery, None);

    ctx.delete();
}

#[sea_orm_macros::test]
pub fn find_by_id_with_result() {
    let ctx = TestContext::new("find_by_id_with_result");
    create_tables(&ctx.db).unwrap();

    let bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&ctx.db)
    .expect("could not insert bakery");

    let result = Bakery::find_by_id(bakery.id.clone().unwrap())
        .one(&ctx.db)
        .unwrap()
        .unwrap();

    assert_eq!(result.id, bakery.id.unwrap());

    ctx.delete();
}

#[sea_orm_macros::test]
pub fn find_all_with_no_result() {
    let ctx = TestContext::new("find_all_with_no_result");
    create_tables(&ctx.db).unwrap();

    let bakeries = Bakery::find().all(&ctx.db).unwrap();
    assert_eq!(bakeries.len(), 0);

    ctx.delete();
}

#[sea_orm_macros::test]
pub fn find_all_with_result() {
    let ctx = TestContext::new("find_all_with_result");
    create_tables(&ctx.db).unwrap();

    let _ = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&ctx.db)
    .expect("could not insert bakery");

    let _ = bakery::ActiveModel {
        name: Set("Top Bakery".to_owned()),
        profit_margin: Set(15.0),
        ..Default::default()
    }
    .save(&ctx.db)
    .expect("could not insert bakery");

    let bakeries = Bakery::find().all(&ctx.db).unwrap();

    assert_eq!(bakeries.len(), 2);

    ctx.delete();
}

#[sea_orm_macros::test]
pub fn find_all_filter_no_result() {
    let ctx = TestContext::new("find_all_filter_no_result");
    create_tables(&ctx.db).unwrap();

    let _ = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&ctx.db)
    .expect("could not insert bakery");

    let _ = bakery::ActiveModel {
        name: Set("Top Bakery".to_owned()),
        profit_margin: Set(15.0),
        ..Default::default()
    }
    .save(&ctx.db)
    .expect("could not insert bakery");

    let bakeries = Bakery::find()
        .filter(bakery::Column::Name.contains("Good"))
        .all(&ctx.db)
        .unwrap();

    assert_eq!(bakeries.len(), 0);

    ctx.delete();
}

#[sea_orm_macros::test]
pub fn find_all_filter_with_results() {
    let ctx = TestContext::new("find_all_filter_with_results");
    create_tables(&ctx.db).unwrap();

    let _ = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&ctx.db)
    .expect("could not insert bakery");

    let _ = bakery::ActiveModel {
        name: Set("Top Bakery".to_owned()),
        profit_margin: Set(15.0),
        ..Default::default()
    }
    .save(&ctx.db)
    .expect("could not insert bakery");

    let bakeries = Bakery::find()
        .filter(bakery::Column::Name.contains("Bakery"))
        .all(&ctx.db)
        .unwrap();

    assert_eq!(bakeries.len(), 2);

    ctx.delete();
}

#[sea_orm_macros::test]
pub fn select_only_exclude_option_fields() {
    let ctx = TestContext::new("select_only_exclude_option_fields");
    create_tables(&ctx.db).unwrap();

    let _ = customer::ActiveModel {
        name: Set("Alice".to_owned()),
        notes: Set(Some("Want to communicate with Bob".to_owned())),
        ..Default::default()
    }
    .save(&ctx.db)
    .expect("could not insert customer");

    let _ = customer::ActiveModel {
        name: Set("Bob".to_owned()),
        notes: Set(Some("Just listening".to_owned())),
        ..Default::default()
    }
    .save(&ctx.db)
    .expect("could not insert customer");

    let _ = customer::ActiveModel {
        name: Set("Sam".to_owned()),
        notes: Set(None),
        ..Default::default()
    }
    .insert(&ctx.db)
    .expect("could not insert customer");

    let customers = Customer::find()
        .select_only()
        .column(customer::Column::Id)
        .column(customer::Column::Name)
        .all(&ctx.db)
        .unwrap();

    assert_eq!(customers.len(), 3);
    assert_eq!(customers[0].notes, None);
    assert_eq!(customers[1].notes, None);
    assert_eq!(customers[2].notes, None);

    let sam = bakery_dense::customer::Entity::find()
        .filter(bakery_dense::customer::COLUMN.name.eq("Sam"))
        .one(&ctx.db)
        .unwrap()
        .unwrap();

    let _: sea_orm::StringColumnNullable<bakery_dense::customer::Entity> =
        bakery_dense::customer::COLUMN.notes;

    assert_eq!(
        sam,
        bakery_dense::customer::Entity::find()
            .filter(
                bakery_dense::customer::COLUMN
                    .notes
                    .eq(Option::<String>::None)
            )
            .one(&ctx.db)
            .unwrap()
            .unwrap()
    );

    ctx.delete();
}
