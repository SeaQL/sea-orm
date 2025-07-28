#![allow(unused_imports, dead_code)]

pub mod common;
#[cfg(feature = "rbac")]
mod rbac;

pub use common::{TestContext, bakery_chain::*, setup::*};
use sea_orm::{ConnectionTrait, DbConn, DbErr, EntityTrait, Set};

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
    // manager can't create bakery
    matches!(
        Bakery::insert(bakery::ActiveModel::default())
            .exec(&db.restricted_for(manager)?)
            .await,
        Err(DbErr::AccessDenied { .. })
    );
    for user_id in [1, 2, 3] {
        // anyone can read bakery
        let db = db.restricted_for(RbacUserId(user_id))?;

        let bakery = Bakery::find().one(&db).await?.unwrap();
        assert_eq!(bakery.name, "SeaSide Bakery");
    }

    Ok(())
}
