#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{TestContext, bakery_chain::*, setup::*};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DbConn, DbErr, EntityName, EntityTrait, IntoActiveModel, NotSet,
    QueryFilter, Set, TransactionTrait, entity::prelude::ChronoUtc,
};

#[sea_orm_macros::test]
#[cfg(feature = "rbac")]
fn main() {
    let ctx = TestContext::new("bakery_chain_rbac_tests");
    create_tables(&ctx.db).unwrap();
    sea_orm::rbac::schema::create_tables(&ctx.db, Default::default()).unwrap();
    rbac_setup(&ctx.db).unwrap();
    crud_tests(&ctx.db).unwrap();
    ctx.delete();
}

#[cfg(feature = "rbac")]
fn rbac_setup(db: &DbConn) -> Result<(), DbErr> {
    use sea_orm::rbac::{RbacAddRoleHierarchy, RbacContext};

    let mut context = RbacContext::load(db)?;

    let tables = [
        baker::Entity.table_name(),
        bakery::Entity.table_name(),
        cake::Entity.table_name(),
        cakes_bakers::Entity.table_name(),
        customer::Entity.table_name(),
        lineitem::Entity.table_name(),
        order::Entity.table_name(),
        "*", // WILDCARD
    ];

    context.add_tables(db, &tables)?;

    context.add_crud_permissions(db)?;

    context.add_roles(db, &["admin", "manager", "public"])?;

    context.assign_user_role(db, &[(1, "admin"), (2, "manager"), (3, "public")])?;

    // public can select everything
    context.add_role_permissions(db, "public", &["select"], &["*"])?;

    // manager can create / update everything except bakery
    context.add_role_permissions(
        db,
        "manager",
        &["insert", "update"],
        &tables
            .iter()
            .cloned()
            .filter(|t| !matches!(*t, "bakery" | "*"))
            .collect::<Vec<_>>(),
    )?;

    // manager can delete order
    context.add_role_permissions(db, "manager", &["delete"], &["order", "lineitem"])?;

    // admin can do anything, in addition to public / manager
    context.add_role_permissions(db, "admin", &["delete"], &["*"])?;

    // add permissions to bakery which manager doesn't have
    context.add_role_permissions(db, "admin", &["insert", "update"], &["bakery"])?;

    context.add_role_hierarchy(
        db,
        &[
            RbacAddRoleHierarchy {
                super_role: "admin",
                role: "manager",
            },
            RbacAddRoleHierarchy {
                super_role: "manager",
                role: "public",
            },
        ],
    )?;

    Ok(())
}

#[cfg(feature = "rbac")]
fn crud_tests(db: &DbConn) -> Result<(), DbErr> {
    use sea_orm::{RestrictedConnection, RestrictedTransaction, rbac::RbacUserId};

    db.load_rbac()?;

    let admin = RbacUserId(1);
    let manager = RbacUserId(2);
    let public = RbacUserId(3);

    fn admin_create_bakery(db: RestrictedConnection) -> Result<(), DbErr> {
        // only admin can create bakery
        let seaside_bakery = bakery::ActiveModel {
            name: Set("SeaSide Bakery".to_owned()),
            profit_margin: Set(10.2),
            ..Default::default()
        };
        let res = Bakery::insert(seaside_bakery).exec(&db)?;
        let bakery: Option<bakery::Model> = Bakery::find_by_id(res.last_insert_id).one(&db)?;

        assert_eq!(bakery.unwrap().name, "SeaSide Bakery");
        Ok(())
    }

    admin_create_bakery(db.restricted_for(admin)?)?;

    // manager / public can't create bakery
    for user in [manager, public] {
        assert!(matches!(
            Bakery::insert(bakery::ActiveModel::default()).exec(&db.restricted_for(user)?),
            Err(DbErr::AccessDenied { .. })
        ));
        let txn = db.restricted_for(user)?.begin()?;
        assert!(matches!(
            Bakery::insert(bakery::ActiveModel::default()).exec(&txn),
            Err(DbErr::AccessDenied { .. })
        ));
    }

    // anyone can read bakery
    for user_id in [1, 2, 3] {
        let db = db.restricted_for(RbacUserId(user_id))?;

        let bakery = Bakery::find().one(&db)?.unwrap();
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
        .expect("insert succeeds");

        db.transaction::<_, _, DbErr>(|txn| {
            ({
                cake::Entity::insert(cake::ActiveModel {
                    name: Set("Chocolate".to_owned()),
                    price: Set(3.into()),
                    bakery_id: Set(Some(1)),
                    gluten_free: Set(true),
                    ..Default::default()
                })
                .exec(txn)?;

                Ok(())
            })
        })
        .expect("insert succeeds");

        let txn: RestrictedTransaction = db.begin()?;

        baker::Entity::insert(baker::ActiveModel {
            name: Set("Master Baker".to_owned()),
            contact_details: Set(Default::default()),
            bakery_id: Set(Some(1)),
            ..Default::default()
        })
        .exec(&txn)
        .expect("insert succeeds");

        txn.commit()?;
    }

    assert_eq!(cake::Entity::find().all(db)?.len(), 2);

    // anyone can read cake
    for user_id in [1, 2, 3] {
        let db = db.restricted_for(RbacUserId(user_id))?;

        let cake = cake::Entity::find().one(&db)?.unwrap();
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
        .exec(&db)?;

        customer::Entity::insert(customer::ActiveModel {
            id: Set(12),
            name: Set("Bob".to_owned()),
            notes: Set(None),
        })
        .exec(&db)?;
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
            placed_at: Set(ChronoUtc::now()),
        })
        .exec(&db)?;

        lineitem::Entity::insert(lineitem::ActiveModel {
            id: NotSet,
            price: Set(2.into()),
            quantity: Set(2),
            order_id: Set(101),
            cake_id: Set(1),
        })
        .exec(&db)?;

        let to_insert = lineitem::ActiveModel {
            id: NotSet,
            price: Set(3.into()),
            quantity: Set(3),
            order_id: Set(101),
            cake_id: Set(2),
        };
        let lineitem_id = if db.support_returning() {
            lineitem::Entity::insert(to_insert)
                .exec_with_returning(&db)?
                .id
        } else {
            lineitem::Entity::insert(to_insert)
                .exec(&db)?
                .last_insert_id
        };

        let order_with_items = order::Entity::find()
            .find_with_related(lineitem::Entity)
            .all(&public_db)?;
        assert_eq!(order_with_items[0].1.len(), 2);

        lineitem::Entity::delete_many()
            .filter(lineitem::Column::Id.eq(lineitem_id))
            .exec(&db)?;

        // reject; of course
        assert!(matches!(
            lineitem::Entity::delete_many()
                .filter(lineitem::Column::Id.eq(lineitem_id))
                .exec(&public_db),
            Err(DbErr::AccessDenied { .. })
        ));

        // only 1 line item left
        let order_with_items = order::Entity::find()
            .find_with_related(lineitem::Entity)
            .all(&public_db)?;
        assert_eq!(order_with_items[0].1.len(), 1);
    }

    // manager can update order
    {
        use sea_orm::ActiveModelTrait;

        let db = db.restricted_for(manager)?;

        let lineitem = lineitem::Entity::find_by_id(1).one(&db)?.unwrap();
        assert_eq!(lineitem.quantity, 2);
        let mut lineitem = lineitem.into_active_model();
        lineitem.quantity = Set(3);
        let lineitem = lineitem.save(&db)?;
        assert_eq!(lineitem.quantity.unwrap(), 3);
    }

    // only admin can delete customer
    {
        use sea_orm::ModelTrait;

        let db = db.restricted_for(admin)?;

        let bob = customer::Entity::find_by_id(12).one(&db)?.unwrap();
        assert_eq!(bob.name, "Bob");

        bob.delete(&db)?;
        assert!(customer::Entity::find_by_id(12).one(&db)?.is_none());
    }

    Ok(())
}
