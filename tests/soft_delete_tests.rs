pub mod common;

pub use common::{features::*, setup::*, TestContext};
use pretty_assertions::assert_eq;
use sea_orm::{entity::prelude::*, entity::*, DatabaseConnection, QueryOrder};
use sea_query::{Alias, Expr, Order};

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn main() -> Result<(), DbErr> {
    {
        let ctx = setup().await?;
        soft_delete_and_find(&ctx.db).await?;
    }
    {
        let ctx = setup().await?;
        force_delete_and_find(&ctx.db).await?;
    }

    Ok(())
}

pub async fn setup() -> Result<TestContext, DbErr> {
    let ctx = TestContext::new("soft_delete_tests").await;
    create_tables(&ctx.db).await?;
    insert_models(&ctx.db).await?;
    Ok(ctx)
}

pub async fn insert_models(db: &DatabaseConnection) -> Result<(), DbErr> {
    // Insert users
    let users = [
        user::Model {
            id: 1,
            name: "Chris".to_owned(),
        },
        user::Model {
            id: 2,
            name: "Billy".to_owned(),
        },
    ];
    User::insert_many(users.clone().into_iter().map(|m| m.into_active_model()))
        .exec(db)
        .await?;
    assert_eq!(User::find().all(db).await?, users);

    // Insert access tokens
    let access_tokens = [
        access_token::Model {
            id: 1,
            user_id: 1,
            token: "7a7bba48-d463-4a8e-b68c-06f74baffcac".parse().unwrap(),
            deleted_at: None,
        },
        access_token::Model {
            id: 2,
            user_id: 1,
            token: "89c55f9a-fbff-42bc-871e-72d29c744ab2".parse().unwrap(),
            deleted_at: None,
        },
        access_token::Model {
            id: 3,
            user_id: 2,
            token: "47c55d32-06ce-4025-95bf-aa4171b46aeb".parse().unwrap(),
            deleted_at: None,
        },
        access_token::Model {
            id: 4,
            user_id: 2,
            token: "c9b7f4cf-053a-4a08-a33d-ac0c506e9187".parse().unwrap(),
            deleted_at: None,
        },
    ];
    AccessToken::insert_many(
        access_tokens
            .clone()
            .into_iter()
            .map(|m| m.into_active_model()),
    )
    .exec(db)
    .await?;
    assert_eq!(AccessToken::find().all(db).await?, access_tokens);

    // Insert access logs
    let access_logs = [
        access_log::Model {
            id: 1,
            access_token_id: 1,
            description: "Chris's first login".to_owned(),
        },
        access_log::Model {
            id: 2,
            access_token_id: 2,
            description: "Chris's first login".to_owned(),
        },
        access_log::Model {
            id: 3,
            access_token_id: 3,
            description: "Billy's first login".to_owned(),
        },
        access_log::Model {
            id: 4,
            access_token_id: 4,
            description: "Billy's first login".to_owned(),
        },
    ];
    AccessLog::insert_many(
        access_logs
            .clone()
            .into_iter()
            .map(|m| m.into_active_model()),
    )
    .exec(db)
    .await?;
    assert_eq!(AccessLog::find().all(db).await?, access_logs);

    Ok(())
}

pub async fn soft_delete_and_find(db: &DatabaseConnection) -> Result<(), DbErr> {
    // Soft delete one of access token of Billy with `id` equals to `3`
    AccessToken::find_by_id(3)
        .one(db)
        .await?
        .unwrap()
        .delete(db)
        .await?;
    assert_eq!(AccessToken::find_by_id(3).one(db).await?, None);

    // Normalize all `deleted_at` values
    AccessToken::update_many()
        .col_expr(
            access_token::Column::DeletedAt,
            Expr::value(get_date_time()),
        )
        .filter(access_token::Column::DeletedAt.is_not_null())
        .exec(db)
        .await?;

    // Count access tokens
    assert_eq!(AccessToken::find().count(db).await?, 3);
    assert_eq!(AccessToken::find_deleted().count(db).await?, 1);
    assert_eq!(AccessToken::find_with_deleted().count(db).await?, 4);

    find_related_models(db).await?;
    find_linked_models(db).await?;

    // Restore soft deleted access token
    AccessToken::find_with_deleted()
        .filter(access_token::Column::Id.eq(3))
        .one(db)
        .await?
        .unwrap()
        .restore_deleted(db)
        .await?;

    // Count access tokens
    assert_eq!(AccessToken::find().count(db).await?, 4);
    assert_eq!(AccessToken::find_deleted().count(db).await?, 0);
    assert_eq!(AccessToken::find_with_deleted().count(db).await?, 4);

    Ok(())
}

pub async fn force_delete_and_find(db: &DatabaseConnection) -> Result<(), DbErr> {
    // Force delete one of access token of Billy with `id` equals to `3`
    AccessToken::find_by_id(3)
        .one(db)
        .await?
        .unwrap()
        .delete_force(db)
        .await?;
    assert_eq!(AccessToken::find_by_id(3).one(db).await?, None);

    // Count access tokens
    assert_eq!(AccessToken::find().count(db).await?, 3);
    assert_eq!(AccessToken::find_deleted().count(db).await?, 0);
    assert_eq!(AccessToken::find_with_deleted().count(db).await?, 3);

    find_related_models(db).await?;
    find_linked_models(db).await?;

    Ok(())
}

pub async fn find_related_models(db: &DatabaseConnection) -> Result<(), DbErr> {
    let billy = User::find_by_id(2).one(db).await?.unwrap();

    // List all access tokens of Billy
    assert_eq!(
        billy.find_related(AccessToken).all(db).await?,
        [access_token::Model {
            id: 4,
            user_id: 2,
            token: "c9b7f4cf-053a-4a08-a33d-ac0c506e9187".parse().unwrap(),
            deleted_at: None,
        }]
    );

    // List all users and their access tokens
    assert_eq!(
        User::find()
            .find_also_related(AccessToken)
            .order_by_asc(access_token::Column::Id)
            .all(db)
            .await?,
        [
            (
                user::Model {
                    id: 1,
                    name: "Chris".to_owned(),
                },
                Some(access_token::Model {
                    id: 1,
                    user_id: 1,
                    token: "7a7bba48-d463-4a8e-b68c-06f74baffcac".parse().unwrap(),
                    deleted_at: None,
                })
            ),
            (
                user::Model {
                    id: 1,
                    name: "Chris".to_owned(),
                },
                Some(access_token::Model {
                    id: 2,
                    user_id: 1,
                    token: "89c55f9a-fbff-42bc-871e-72d29c744ab2".parse().unwrap(),
                    deleted_at: None,
                })
            ),
            (
                user::Model {
                    id: 2,
                    name: "Billy".to_owned(),
                },
                Some(access_token::Model {
                    id: 4,
                    user_id: 2,
                    token: "c9b7f4cf-053a-4a08-a33d-ac0c506e9187".parse().unwrap(),
                    deleted_at: None,
                })
            ),
        ]
    );
    assert_eq!(
        User::find()
            .find_with_related(AccessToken)
            .order_by_asc(access_token::Column::Id)
            .all(db)
            .await?,
        [
            (
                user::Model {
                    id: 1,
                    name: "Chris".to_owned(),
                },
                vec![
                    access_token::Model {
                        id: 1,
                        user_id: 1,
                        token: "7a7bba48-d463-4a8e-b68c-06f74baffcac".parse().unwrap(),
                        deleted_at: None,
                    },
                    access_token::Model {
                        id: 2,
                        user_id: 1,
                        token: "89c55f9a-fbff-42bc-871e-72d29c744ab2".parse().unwrap(),
                        deleted_at: None,
                    },
                ]
            ),
            (
                user::Model {
                    id: 2,
                    name: "Billy".to_owned(),
                },
                vec![access_token::Model {
                    id: 4,
                    user_id: 2,
                    token: "c9b7f4cf-053a-4a08-a33d-ac0c506e9187".parse().unwrap(),
                    deleted_at: None,
                }]
            ),
        ]
    );

    Ok(())
}

pub async fn find_linked_models(db: &DatabaseConnection) -> Result<(), DbErr> {
    let billy = User::find_by_id(2).one(db).await?.unwrap();

    // List all access log of Billy
    assert_eq!(
        billy.find_linked(user::AccessLogLink).all(db).await?,
        [access_log::Model {
            id: 4,
            access_token_id: 4,
            description: "Billy's first login".to_owned(),
        }]
    );

    // List all users and their access log
    let mut find_also_linked = User::find().find_also_linked(user::AccessLogLink);
    QueryOrder::query(&mut find_also_linked)
        .order_by((Alias::new("r1"), Alias::new("id")), Order::Asc);
    assert_eq!(
        find_also_linked.all(db).await?,
        [
            (
                user::Model {
                    id: 1,
                    name: "Chris".to_owned(),
                },
                Some(access_log::Model {
                    id: 1,
                    access_token_id: 1,
                    description: "Chris's first login".to_owned(),
                })
            ),
            (
                user::Model {
                    id: 1,
                    name: "Chris".to_owned(),
                },
                Some(access_log::Model {
                    id: 2,
                    access_token_id: 2,
                    description: "Chris's first login".to_owned(),
                })
            ),
            (
                user::Model {
                    id: 2,
                    name: "Billy".to_owned(),
                },
                Some(access_log::Model {
                    id: 4,
                    access_token_id: 4,
                    description: "Billy's first login".to_owned(),
                })
            ),
        ]
    );

    Ok(())
}

fn get_date_time() -> DateTimeWithTimeZone {
    "2022-06-10T16:24:00+00:00".parse().unwrap()
}
