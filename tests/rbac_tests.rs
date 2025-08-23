#![allow(unused_imports, dead_code)]

pub mod common;
#[cfg(feature = "rbac")]
mod rbac;

pub use common::{TestContext, bakery_chain::*, setup::*};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DbConn, DbErr, EntityTrait, IntoActiveModel, NotSet, QueryFilter,
    Set, TransactionTrait,
};

#[sea_orm_macros::test]
#[cfg(feature = "rbac")]
async fn main() {
    let ctx = TestContext::new("bakery_chain_rbac_tests").await;
    create_tables(&ctx.db).await.unwrap();
    sea_orm::rbac::schema::create_tables(&ctx.db).await.unwrap();
    rbac::setup(&ctx.db).await.unwrap();
    crud_tests(&ctx.db).await.unwrap();
    ctx.delete().await;
}

#[cfg(feature = "rbac")]
async fn crud_tests(db: &DbConn) -> Result<(), DbErr> {
    use sea_orm::rbac::RbacUserId;
    let admin = RbacUserId(1);
    let manager = RbacUserId(2);
    let public = RbacUserId(3);

    db.load_rbac().await?;

    {
        // only admin can create bakery
        let db = db.restricted_for(admin)?;

        let seaside_bakery = bakery::ActiveModel {
            name: Set("SeaSide Bakery".to_owned()),
            profit_margin: Set(10.2),
            ..Default::default()
        };
        let res = Bakery::insert(seaside_bakery).exec(&db).await?;
        let bakery: Option<bakery::Model> = Bakery::find_by_id(res.last_insert_id).one(&db).await?;

        assert_eq!(bakery.unwrap().name, "SeaSide Bakery");
    }

    // manager / public can't create bakery
    for user in [manager, public] {
        assert!(matches!(
            Bakery::insert(bakery::ActiveModel::default())
                .exec(&db.restricted_for(user)?)
                .await,
            Err(DbErr::AccessDenied { .. })
        ));
        let txn = db.restricted_for(user)?.begin().await?;
        assert!(matches!(
            Bakery::insert(bakery::ActiveModel::default())
                .exec(&txn)
                .await,
            Err(DbErr::AccessDenied { .. })
        ));
    }

    // anyone can read bakery
    for user_id in [1, 2, 3] {
        let db = db.restricted_for(RbacUserId(user_id))?;

        let bakery = Bakery::find().one(&db).await?.unwrap();
        assert_eq!(bakery.name, "SeaSide Bakery");
    }

    // manager can create cake / baker
    {
        let db = db.restricted_for(manager)?;

        cake::Entity::insert(cake::ActiveModel {
            name: Set("Cheesecake".to_owned()),
            price: Set(2.into()),
            bakery_id: Set(Some(1)),
            gluten_free: Set(false),
            ..Default::default()
        })
        .exec(&db)
        .await
        .expect("insert succeeds");

        db.transaction::<_, _, DbErr>(|txn| {
            Box::pin(async move {
                cake::Entity::insert(cake::ActiveModel {
                    name: Set("Chocolate".to_owned()),
                    price: Set(3.into()),
                    bakery_id: Set(Some(1)),
                    gluten_free: Set(true),
                    ..Default::default()
                })
                .exec(txn)
                .await?;

                Ok(())
            })
        })
        .await
        .expect("insert succeeds");

        let txn = db.begin().await?;

        baker::Entity::insert(baker::ActiveModel {
            name: Set("Master Baker".to_owned()),
            contact_details: Set(Default::default()),
            bakery_id: Set(Some(1)),
            ..Default::default()
        })
        .exec(&txn)
        .await
        .expect("insert succeeds");

        txn.commit().await?;
    }

    assert_eq!(cake::Entity::find().all(db).await?.len(), 2);

    // anyone can read cake
    for user_id in [1, 2, 3] {
        let db = db.restricted_for(RbacUserId(user_id))?;

        let cake = cake::Entity::find().one(&db).await?.unwrap();
        assert_eq!(cake.name, "Cheesecake");
    }

    // admin can create customer
    {
        let db = db.restricted_for(admin)?;

        customer::Entity::insert(customer::ActiveModel {
            id: Set(11),
            name: Set("Alice".to_owned()),
            notes: Set(None),
        })
        .exec(&db)
        .await?;

        customer::Entity::insert(customer::ActiveModel {
            id: Set(12),
            name: Set("Bob".to_owned()),
            notes: Set(None),
        })
        .exec(&db)
        .await?;
    }

    // manager can create / delete order
    {
        let public_db = db.restricted_for(public)?;
        let db = db.restricted_for(manager)?;

        order::Entity::insert(order::ActiveModel {
            id: Set(101),
            total: Set(10.into()),
            bakery_id: Set(1),
            customer_id: Set(11),
            placed_at: Set(Default::default()),
        })
        .exec(&db)
        .await?;

        lineitem::Entity::insert(lineitem::ActiveModel {
            id: NotSet,
            price: Set(2.into()),
            quantity: Set(2),
            order_id: Set(101),
            cake_id: Set(1),
        })
        .exec(&db)
        .await?;

        let to_insert = lineitem::ActiveModel {
            id: NotSet,
            price: Set(3.into()),
            quantity: Set(3),
            order_id: Set(101),
            cake_id: Set(2),
        };
        let lineitem_id = if db.support_returning() {
            lineitem::Entity::insert(to_insert)
                .exec_with_returning(&db)
                .await?
                .id
        } else {
            lineitem::Entity::insert(to_insert)
                .exec(&db)
                .await?
                .last_insert_id
        };

        let order_with_items = order::Entity::find()
            .find_with_related(lineitem::Entity)
            .all(&public_db)
            .await?;
        assert_eq!(order_with_items[0].1.len(), 2);

        lineitem::Entity::delete_many()
            .filter(lineitem::Column::Id.eq(lineitem_id))
            .exec(&db)
            .await?;

        // reject; of course
        assert!(matches!(
            lineitem::Entity::delete_many()
                .filter(lineitem::Column::Id.eq(lineitem_id))
                .exec(&public_db)
                .await,
            Err(DbErr::AccessDenied { .. })
        ));

        // only 1 line item left
        let order_with_items = order::Entity::find()
            .find_with_related(lineitem::Entity)
            .all(&public_db)
            .await?;
        assert_eq!(order_with_items[0].1.len(), 1);
    }

    // manager can update order
    {
        use sea_orm::ActiveModelTrait;

        let db = db.restricted_for(manager)?;

        let lineitem = lineitem::Entity::find_by_id(1).one(&db).await?.unwrap();
        assert_eq!(lineitem.quantity, 2);
        let mut lineitem = lineitem.into_active_model();
        lineitem.quantity = Set(3);
        let lineitem = lineitem.save(&db).await?;
        assert_eq!(lineitem.quantity.unwrap(), 3);
    }

    // only admin can delete customer
    {
        use sea_orm::ModelTrait;

        let db = db.restricted_for(admin)?;

        let bob = customer::Entity::find_by_id(12).one(&db).await?.unwrap();
        assert_eq!(bob.name, "Bob");

        bob.delete(&db).await?;
        assert!(customer::Entity::find_by_id(12).one(&db).await?.is_none());
    }

    Ok(())
}
