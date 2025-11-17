#![allow(unused_imports, dead_code)]

mod common;

use crate::common::TestContext;
use sea_orm::{Database, DbConn, DbErr, entity::*, prelude::*, query::*, tests_cfg::*};

#[sea_orm_macros::test]
async fn test_active_model_ex() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_active_model_ex").await;
    let db = &ctx.db;

    db.get_schema_builder()
        .register(user::Entity)
        .register(profile::Entity)
        .register(post::Entity)
        .register(post_tag::Entity)
        .register(tag::Entity)
        .apply(db)
        .await?;

    tracing::info!("save a new user");
    let user = user::ActiveModel {
        id: NotSet,
        name: Set("Alice".into()),
        email: Set("@1".into()),
    }
    .save(db)
    .await?;

    assert_eq!(user.id, Unchanged(1));

    let mut post = post::ActiveModel {
        title: Set("post 1".into()),
        ..Default::default()
    }
    .into_ex();

    tracing::info!("save a post with an existing user");
    post.author = HasOneModel::set(user.into_ex());
    let post = post.save(db).await?;

    assert_eq!(
        post,
        post::ActiveModelEx {
            id: Unchanged(1),
            user_id: Unchanged(1),
            title: Unchanged("post 1".into()),
            author: HasOneModel::set(user::ActiveModelEx {
                id: Unchanged(1),
                name: Unchanged("Alice".into()),
                email: Unchanged("@1".into()),
                profile: HasOneModel::NotSet,
                posts: HasManyModel::NotSet,
            }),
            comments: HasManyModel::NotSet,
            tags: HasManyModel::NotSet,
        }
    );

    tracing::info!("save a post with a new user");
    let post = post::ActiveModelEx {
        title: Set("post 2".into()),
        author: HasOneModel::set(user::ActiveModelEx {
            name: Set("Bob".into()),
            email: Set("@2".into()),
            ..Default::default()
        }),
        ..Default::default()
    }
    .save(db)
    .await?;

    assert_eq!(
        post,
        post::ActiveModelEx {
            id: Unchanged(2),
            user_id: Unchanged(2),
            title: Unchanged("post 2".into()),
            author: HasOneModel::set(user::ActiveModelEx {
                id: Unchanged(2),
                name: Unchanged("Bob".into()),
                email: Unchanged("@2".into()),
                ..Default::default()
            }),
            ..Default::default()
        }
    );

    tracing::info!("save a new user with a new profile");
    let user = user::ActiveModelEx {
        name: Set("Sam".into()),
        email: Set("@3".into()),
        profile: HasOneModel::set(profile::ActiveModelEx {
            picture: Set("Sam.jpg".into()),
            ..Default::default()
        }),
        ..Default::default()
    }
    .save(db)
    .await?;

    assert_eq!(
        user,
        user::ActiveModelEx {
            id: Unchanged(3),
            name: Unchanged("Sam".into()),
            email: Unchanged("@3".into()),
            profile: HasOneModel::set(profile::ActiveModelEx {
                id: Unchanged(1),
                picture: Unchanged("Sam.jpg".into()),
                user_id: Unchanged(3),
                user: HasOneModel::NotSet,
            }),
            ..Default::default()
        }
    );

    tracing::info!("save a new user with a new profile and 2 posts");
    let mut user = user::ActiveModelEx {
        id: NotSet,
        name: Set("Alan".into()),
        email: Set("@4".into()),
        profile: HasOneModel::set(profile::ActiveModelEx {
            picture: Set("Alan.jpg".into()),
            ..Default::default()
        }),
        posts: HasManyModel::Append(vec![
            post::ActiveModelEx {
                title: Set("post 3".into()),
                ..Default::default()
            },
            post::ActiveModelEx {
                title: Set("post 4".into()),
                ..Default::default()
            },
        ]),
    }
    .save(db)
    .await?;

    assert_eq!(
        user,
        user::ActiveModelEx {
            id: Unchanged(4),
            name: Unchanged("Alan".into()),
            email: Unchanged("@4".into()),
            profile: HasOneModel::set(profile::ActiveModelEx {
                id: Unchanged(2),
                picture: Unchanged("Alan.jpg".into()),
                user_id: Unchanged(4),
                user: HasOneModel::NotSet,
            }),
            posts: HasManyModel::Append(vec![
                post::ActiveModelEx {
                    id: Unchanged(3),
                    user_id: Unchanged(4),
                    title: Unchanged("post 3".into()),
                    ..Default::default()
                },
                post::ActiveModelEx {
                    id: Unchanged(4),
                    user_id: Unchanged(4),
                    title: Unchanged("post 4".into()),
                    ..Default::default()
                },
            ]),
        }
    );

    let posts = user.find_related_of(user.posts.as_slice()).all(db).await?;
    assert_eq!(posts.len(), 2);
    assert_eq!(posts[0].id, 3);
    assert_eq!(posts[1].id, 4);

    tracing::info!("replace posts of user: delete 3,4; insert 5");
    user.posts = HasManyModel::Replace(vec![post::ActiveModelEx {
        title: Set("post 5".into()),
        ..Default::default()
    }]);

    let mut user = user.save(db).await?;

    let posts = user.find_related_of(user.posts.as_slice()).all(db).await?;
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].id, 5);

    tracing::info!("add new post to user: insert 6");
    user.posts = HasManyModel::Append(vec![post::ActiveModelEx {
        title: Set("post 6".into()),
        ..Default::default()
    }]);

    let mut user = user.save(db).await?;

    let posts = user.find_related_of(user.posts.as_slice()).all(db).await?;
    assert_eq!(posts.len(), 2);
    assert_eq!(posts[0].id, 5);
    assert_eq!(posts[1].id, 6);

    tracing::info!("update user profile and delete all posts");
    user.profile.as_mut().unwrap().picture = Set("Alan2.jpg".into());
    user.posts = HasManyModel::Replace(vec![]);

    user.save(db).await?;

    let user = user::Entity::load()
        .filter_by_id(4)
        .with(profile::Entity)
        .with(post::Entity)
        .one(db)
        .await?
        .unwrap();

    assert_eq!(
        user,
        user::ModelEx {
            id: 4,
            name: "Alan".into(),
            email: "@4".into(),
            profile: HasOne::loaded(profile::Model {
                id: 2,
                picture: "Alan2.jpg".into(),
                user_id: 4,
            }),
            posts: HasMany::Loaded(vec![]),
        }
    );

    tracing::info!("insert one tag for later use");
    let day = tag::ActiveModel {
        tag: Set("day".into()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    tracing::info!("insert new post and set 2 tags");
    let mut post = post::ActiveModelEx {
        id: NotSet,
        user_id: NotSet,
        title: Set("post 7".into()),
        author: HasOneModel::set(user.into_active_model()),
        comments: HasManyModel::NotSet,
        tags: HasManyModel::Replace(vec![
            day.into_active_model().into(),
            tag::ActiveModel {
                id: NotSet,
                tag: Set("pet".into()),
            }
            .into(),
        ]),
    }
    .save(db)
    .await?;

    assert_eq!(
        post,
        post::ActiveModelEx {
            id: Unchanged(7),
            user_id: Unchanged(4),
            title: Unchanged("post 7".into()),
            author: HasOneModel::set(user::ActiveModelEx {
                id: Unchanged(4),
                name: Unchanged("Alan".into()),
                email: Unchanged("@4".into()),
                profile: HasOneModel::set(profile::ActiveModelEx {
                    id: Unchanged(2),
                    picture: Unchanged("Alan2.jpg".into()),
                    user_id: Unchanged(4),
                    user: HasOneModel::NotSet,
                }),
                posts: HasManyModel::Append(vec![]),
            }),
            comments: HasManyModel::NotSet,
            tags: HasManyModel::Replace(vec![
                tag::ActiveModel {
                    id: Unchanged(1),
                    tag: Unchanged("day".into()),
                }
                .into(),
                tag::ActiveModel {
                    id: Unchanged(2),
                    tag: Unchanged("pet".into()),
                }
                .into(),
            ]),
        }
    );

    tracing::info!("get back the post and tags");
    let post_7 = post::Entity::load()
        .filter_by_id(7)
        .with(tag::Entity)
        .one(db)
        .await?
        .unwrap();

    assert_eq!(post_7.id, 7);
    assert_eq!(post_7.tags.len(), 2);
    assert_eq!(post_7.tags[0].tag, "day");
    assert_eq!(post_7.tags[1].tag, "pet");

    tracing::info!("update user profile through post");
    post.author
        .as_mut()
        .unwrap()
        .profile
        .as_mut()
        .unwrap()
        .picture = Set("Alan3.jpg".into());
    let mut post = post.save(db).await?;
    assert_eq!(
        profile::Entity::find_by_id(2)
            .one(db)
            .await?
            .unwrap()
            .picture,
        "Alan3.jpg"
    );

    tracing::info!("replace post tags: remove tag 1 add tag 3");
    post.tags = HasManyModel::Replace(vec![
        tag::ActiveModel {
            id: NotSet, // new tag
            tag: Set("food".into()),
        }
        .into(),
        tag::ActiveModel {
            id: Unchanged(2), // retain
            tag: Unchanged("pet".into()),
        }
        .into(),
    ]);
    let mut post = post.save(db).await?;

    tracing::info!("get back the post and tags");
    let post_7 = post::Entity::load()
        .filter_by_id(7)
        .with(tag::Entity)
        .one(db)
        .await?
        .unwrap();

    assert_eq!(post_7.id, 7);
    assert_eq!(post_7.tags.len(), 2);
    assert_eq!(post_7.tags[0].tag, "pet");
    assert_eq!(post_7.tags[1].tag, "food");

    tracing::info!("add new tag to post");
    post.tags = HasManyModel::Append(vec![
        tag::ActiveModel {
            id: NotSet, // new tag
            tag: Set("sunny".into()),
        }
        .into(),
    ]);
    post.save(db).await?;

    tracing::info!("get back the post and tags");
    let post_7 = post::Entity::load()
        .filter_by_id(7)
        .with(tag::Entity)
        .one(db)
        .await?
        .unwrap();

    assert_eq!(post_7.id, 7);
    assert_eq!(post_7.tags.len(), 3);
    assert_eq!(post_7.tags[0].tag, "pet");
    assert_eq!(post_7.tags[1].tag, "food");
    assert_eq!(post_7.tags[2].tag, "sunny");

    Ok(())
}
