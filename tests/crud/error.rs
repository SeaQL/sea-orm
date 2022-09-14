pub use super::*;
use rust_decimal_macros::dec;
use sea_orm::error::*;
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
use sqlx::Error;
use uuid::Uuid;

pub async fn test_cake_error_sqlx(db: &DbConn) {
    let mud_cake = cake::ActiveModel {
        name: Set("Moldy Cake".to_owned()),
        price: Set(dec!(10.25)),
        gluten_free: Set(false),
        serial: Set(Uuid::new_v4()),
        bakery_id: Set(None),
        ..Default::default()
    };

    let cake = mud_cake.save(db).await.expect("could not insert cake");

    // if compiling without sqlx, this assignment will complain,
    // but the whole test is useless in that case anyway.
    #[allow(unused_variables)]
    let error: DbErr = cake
        .into_active_model()
        .insert(db)
        .await
        .expect_err("inserting should fail due to duplicate primary key");

    #[cfg(any(
        feature = "sqlx-mysql",
        feature = "sqlx-sqlite",
        feature = "sqlx-postgres"
    ))]
    match error {
        DbErr::UniqueConstraintViolation(RuntimeErr::SqlxError(error)) => match error {
            Error::Database(e) => {
                #[cfg(feature = "sqlx-mysql")]
                assert_eq!(e.code().unwrap(), "23000");
                #[cfg(feature = "sqlx-sqlite")]
                assert_eq!(e.code().unwrap(), "1555");
                #[cfg(feature = "sqlx-postgres")]
                assert_eq!(e.code().unwrap(), "23505");
            }
            _ => panic!("Unexpected sqlx-error kind {:?}", error),
        },
        _ => panic!("Unexpected Error kind {:?}", error),
    }
}
