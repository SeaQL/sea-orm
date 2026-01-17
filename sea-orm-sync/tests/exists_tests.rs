#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{TestContext, bakery_chain::*, setup::*};
pub use sea_orm::entity::*;
pub use sea_orm::{ConnectionTrait, QueryFilter, QueryOrder, QuerySelect, SelectExt};

#[sea_orm_macros::test]
pub fn exists_with_no_result() {
    let ctx = TestContext::new("exists_with_no_result");
    create_tables(&ctx.db).unwrap();

    let exists = Bakery::find().exists(&ctx.db).unwrap();
    assert_eq!(exists, false);

    ctx.delete();
}

#[sea_orm_macros::test]
pub fn exists_with_result() {
    let ctx = TestContext::new("exists_with_result");
    create_tables(&ctx.db).unwrap();

    let _bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&ctx.db)
    .expect("could not insert bakery");

    let exists = Bakery::find().exists(&ctx.db).unwrap();
    assert_eq!(exists, true);

    ctx.delete();
}

#[sea_orm_macros::test]
pub fn exists_with_filter_no_result() {
    let ctx = TestContext::new("exists_with_filter_no_result");
    create_tables(&ctx.db).unwrap();

    let _bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&ctx.db)
    .expect("could not insert bakery");

    let exists = Bakery::find()
        .filter(bakery::Column::Name.contains("Nonexistent"))
        .exists(&ctx.db)
        .unwrap();
    assert_eq!(exists, false);

    ctx.delete();
}

#[sea_orm_macros::test]
pub fn exists_with_filter_has_result() {
    let ctx = TestContext::new("exists_with_filter_has_result");
    create_tables(&ctx.db).unwrap();

    let _bakery1 = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&ctx.db)
    .expect("could not insert bakery");

    let _bakery2 = bakery::ActiveModel {
        name: Set("Top Bakery".to_owned()),
        profit_margin: Set(15.0),
        ..Default::default()
    }
    .save(&ctx.db)
    .expect("could not insert bakery");

    let exists = Bakery::find()
        .filter(bakery::Column::Name.contains("SeaSide"))
        .exists(&ctx.db)
        .unwrap();
    assert_eq!(exists, true);

    ctx.delete();
}

#[sea_orm_macros::test]
pub fn exists_with_complex_query() {
    let ctx = TestContext::new("exists_with_complex_query");
    create_tables(&ctx.db).unwrap();

    let _bakery1 = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&ctx.db)
    .expect("could not insert bakery");

    let _bakery2 = bakery::ActiveModel {
        name: Set("Top Bakery".to_owned()),
        profit_margin: Set(15.0),
        ..Default::default()
    }
    .save(&ctx.db)
    .expect("could not insert bakery");

    let _bakery3 = bakery::ActiveModel {
        name: Set("Low Profit Bakery".to_owned()),
        profit_margin: Set(5.0),
        ..Default::default()
    }
    .save(&ctx.db)
    .expect("could not insert bakery");

    // Test with complex filter - exists bakery with profit margin > 12
    let exists = Bakery::find()
        .filter(bakery::Column::ProfitMargin.gt(12.0))
        .exists(&ctx.db)
        .unwrap();
    assert_eq!(exists, true);

    // Test with complex filter - exists bakery with profit margin > 20
    let exists = Bakery::find()
        .filter(bakery::Column::ProfitMargin.gt(20.0))
        .exists(&ctx.db)
        .unwrap();
    assert_eq!(exists, false);

    ctx.delete();
}

#[sea_orm_macros::test]
pub fn exists_with_joins() {
    let ctx = TestContext::new("exists_with_joins");
    create_tables(&ctx.db).unwrap();

    let bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&ctx.db)
    .expect("could not insert bakery");

    let _cake = cake::ActiveModel {
        name: Set("Chocolate Cake".to_owned()),
        price: Set(rust_dec(12.50)), // 12.50
        bakery_id: Set(Some(bakery.id.unwrap())),
        gluten_free: Set(false),
        serial: Set(uuid::Uuid::new_v4()),
        ..Default::default()
    }
    .save(&ctx.db)
    .expect("could not insert cake");

    // Test exists with join - exists cake from a specific bakery
    let exists = Cake::find()
        .inner_join(Bakery)
        .filter(bakery::Column::Name.eq("SeaSide Bakery"))
        .exists(&ctx.db)
        .unwrap();
    assert_eq!(exists, true);

    // Test exists with join - no cake from non-existent bakery
    let exists = Cake::find()
        .inner_join(Bakery)
        .filter(bakery::Column::Name.eq("Non-existent Bakery"))
        .exists(&ctx.db)
        .unwrap();
    assert_eq!(exists, false);

    ctx.delete();
}

#[sea_orm_macros::test]
pub fn exists_with_ordering() {
    let ctx = TestContext::new("exists_with_ordering");
    create_tables(&ctx.db).unwrap();

    let _bakery1 = bakery::ActiveModel {
        name: Set("A Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&ctx.db)
    .expect("could not insert bakery");

    let _bakery2 = bakery::ActiveModel {
        name: Set("Z Bakery".to_owned()),
        profit_margin: Set(15.0),
        ..Default::default()
    }
    .save(&ctx.db)
    .expect("could not insert bakery");

    // Test exists with order by - should still work and return true
    let exists = Bakery::find()
        .order_by_asc(bakery::Column::Name)
        .exists(&ctx.db)
        .unwrap();
    assert_eq!(exists, true);

    ctx.delete();
}

#[sea_orm_macros::test]
pub fn exists_with_limit_offset() {
    let ctx = TestContext::new("exists_with_limit_offset");
    create_tables(&ctx.db).unwrap();

    // Insert multiple bakeries
    for i in 1..=5 {
        let _bakery = bakery::ActiveModel {
            name: Set(format!("Bakery {}", i)),
            profit_margin: Set(10.0 + (i as f64)),
            ..Default::default()
        }
        .save(&ctx.db)
        .expect("could not insert bakery");
    }

    // Test exists with limit - should still find records
    let exists = Bakery::find().limit(2).exists(&ctx.db).unwrap();
    assert_eq!(exists, true);

    // Test exists with offset - should still find records
    let exists = Bakery::find().offset(3).exists(&ctx.db).unwrap();
    assert_eq!(exists, true);

    // Test exists with limit and offset - exists() checks for existence regardless of offset
    // This is the expected behavior since exists() is optimized to check if ANY record exists
    let exists = Bakery::find()
        .offset(10) // Beyond all records
        .limit(1)
        .exists(&ctx.db)
        .unwrap();
    assert_eq!(exists, true); // exists() ignores offset for performance optimization

    ctx.delete();
}
