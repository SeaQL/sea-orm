use super::*;
use crate::common::TestContext;
use sea_orm::{NotSet, Set, prelude::*};

pub fn init_1(ctx: &TestContext, link: bool) {
    bakery::Entity::insert(bakery::ActiveModel {
        id: Set(42),
        name: Set("cool little bakery".to_string()),
        profit_margin: Set(4.1),
    })
    .exec(&ctx.db)
    .expect("insert succeeds");

    cake::Entity::insert(cake::ActiveModel {
        id: Set(13),
        name: Set("Cheesecake".to_owned()),
        price: Set(2.into()),
        bakery_id: Set(if link { Some(42) } else { None }),
        gluten_free: Set(false),
        ..Default::default()
    })
    .exec(&ctx.db)
    .expect("insert succeeds");

    cake::Entity::insert(cake::ActiveModel {
        id: Set(15),
        name: Set("Chocolate".to_owned()),
        price: Set(3.into()),
        bakery_id: Set(if link { Some(42) } else { None }),
        gluten_free: Set(true),
        ..Default::default()
    })
    .exec(&ctx.db)
    .expect("insert succeeds");

    baker::Entity::insert(baker::ActiveModel {
        id: Set(22),
        name: Set("Master Baker".to_owned()),
        contact_details: Set(Json::Null),
        bakery_id: Set(if link { Some(42) } else { None }),
    })
    .exec(&ctx.db)
    .expect("insert succeeds");

    if link {
        cakes_bakers::Entity::insert(cakes_bakers::ActiveModel {
            cake_id: Set(13),
            baker_id: Set(22),
        })
        .exec(&ctx.db)
        .expect("insert succeeds");

        customer::Entity::insert(customer::ActiveModel {
            id: Set(11),
            name: Set("Bob".to_owned()),
            notes: Set(Some("Sweet tooth".to_owned())),
        })
        .exec(&ctx.db)
        .expect("insert succeeds");

        order::Entity::insert(order::ActiveModel {
            id: Set(101),
            total: Set(10.into()),
            bakery_id: Set(42),
            customer_id: Set(11),
            placed_at: Set("2020-01-01T00:00:00Z".parse().unwrap()),
        })
        .exec(&ctx.db)
        .expect("insert succeeds");

        lineitem::Entity::insert(lineitem::ActiveModel {
            id: NotSet,
            price: Set(2.into()),
            quantity: Set(2),
            order_id: Set(101),
            cake_id: Set(13),
        })
        .exec(&ctx.db)
        .expect("insert succeeds");

        lineitem::Entity::insert(lineitem::ActiveModel {
            id: NotSet,
            price: Set(3.into()),
            quantity: Set(2),
            order_id: Set(101),
            cake_id: Set(15),
        })
        .exec(&ctx.db)
        .expect("insert succeeds");
    }
}

pub fn init_2(ctx: &TestContext) -> Result<(), DbErr> {
    let db = &ctx.db;

    let bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    };
    let sea = Bakery::insert(bakery).exec(db)?.last_insert_id;

    let bakery = bakery::ActiveModel {
        name: Set("LakeSide Bakery".to_owned()),
        profit_margin: Set(5.8),
        ..Default::default()
    };
    let lake = Bakery::insert(bakery).exec(db)?.last_insert_id;

    let alice = baker::ActiveModel {
        name: Set("Alice".to_owned()),
        contact_details: Set("+44 15273388".into()),
        bakery_id: Set(Some(sea)),
        ..Default::default()
    };
    let alice = Baker::insert(alice).exec(db)?.last_insert_id;

    let bob = baker::ActiveModel {
        name: Set("Bob".to_owned()),
        contact_details: Set("+852 12345678".into()),
        bakery_id: Set(Some(lake)),
        ..Default::default()
    };
    let bob = Baker::insert(bob).exec(db)?.last_insert_id;

    let cake = cake::ActiveModel {
        name: Set("Chocolate Cake".to_owned()),
        price: Set("10.25".parse().unwrap()),
        gluten_free: Set(false),
        bakery_id: Set(Some(sea)),
        ..Default::default()
    };
    let choco = Cake::insert(cake).exec(db)?.last_insert_id;

    let cake = cake::ActiveModel {
        name: Set("Double Chocolate".to_owned()),
        price: Set("12.5".parse().unwrap()),
        gluten_free: Set(false),
        bakery_id: Set(Some(sea)),
        ..Default::default()
    };
    let double_1 = Cake::insert(cake.clone()).exec(db)?.last_insert_id;

    let mut cake = cake::ActiveModel {
        name: Set("Lemon Cake".to_owned()),
        price: Set("8.8".parse().unwrap()),
        gluten_free: Set(true),
        bakery_id: Set(Some(sea)),
        ..Default::default()
    };
    let lemon_1 = Cake::insert(cake.clone()).exec(db)?.last_insert_id;
    cake.bakery_id = Set(Some(lake));
    let _lemon_2 = Cake::insert(cake).exec(db)?.last_insert_id;

    let cake = cake::ActiveModel {
        name: Set("Strawberry Cake".to_owned()),
        price: Set("9.9".parse().unwrap()),
        gluten_free: Set(false),
        bakery_id: Set(Some(lake)),
        ..Default::default()
    };
    let straw_2 = Cake::insert(cake).exec(db)?.last_insert_id;

    let cake = cake::ActiveModel {
        name: Set("Orange Cake".to_owned()),
        price: Set("6.5".parse().unwrap()),
        gluten_free: Set(true),
        bakery_id: Set(Some(lake)),
        ..Default::default()
    };
    let orange = Cake::insert(cake).exec(db)?.last_insert_id;

    let mut cake = cake::ActiveModel {
        name: Set("New York Cheese".to_owned()),
        price: Set("12.5".parse().unwrap()),
        gluten_free: Set(false),
        bakery_id: Set(Some(sea)),
        ..Default::default()
    };
    let cheese_1 = Cake::insert(cake.clone()).exec(db)?.last_insert_id;
    cake.bakery_id = Set(Some(lake));
    let cheese_2 = Cake::insert(cake).exec(db)?.last_insert_id;

    let rel = cakes_bakers::ActiveModel {
        cake_id: Set(choco),
        baker_id: Set(alice),
    };
    CakesBakers::insert(rel).exec(db)?;

    let rel = cakes_bakers::ActiveModel {
        cake_id: Set(double_1),
        baker_id: Set(alice),
    };
    CakesBakers::insert(rel).exec(db)?;

    let rel = cakes_bakers::ActiveModel {
        cake_id: Set(lemon_1),
        baker_id: Set(alice),
    };
    CakesBakers::insert(rel).exec(db)?;
    let rel = cakes_bakers::ActiveModel {
        cake_id: Set(lemon_1),
        baker_id: Set(bob),
    };
    CakesBakers::insert(rel).exec(db)?;

    let rel = cakes_bakers::ActiveModel {
        cake_id: Set(straw_2),
        baker_id: Set(bob),
    };
    CakesBakers::insert(rel).exec(db)?;

    let rel = cakes_bakers::ActiveModel {
        cake_id: Set(orange),
        baker_id: Set(bob),
    };
    CakesBakers::insert(rel).exec(db)?;

    let rel = cakes_bakers::ActiveModel {
        cake_id: Set(cheese_1),
        baker_id: Set(alice),
    };
    CakesBakers::insert(rel).exec(db)?;
    let rel = cakes_bakers::ActiveModel {
        cake_id: Set(cheese_1),
        baker_id: Set(bob),
    };
    CakesBakers::insert(rel).exec(db)?;
    let rel = cakes_bakers::ActiveModel {
        cake_id: Set(cheese_2),
        baker_id: Set(alice),
    };
    CakesBakers::insert(rel).exec(db)?;
    let rel = cakes_bakers::ActiveModel {
        cake_id: Set(cheese_2),
        baker_id: Set(bob),
    };
    CakesBakers::insert(rel).exec(db)?;

    Ok(())
}
