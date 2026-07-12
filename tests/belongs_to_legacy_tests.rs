#![allow(unused_imports, dead_code)]
//! Backward-compatibility ("legacy") suite: `belongs_to` relations declared with the
//! `HasOne<E>` field type instead of the recommended `BelongsTo<E>`.
//!
//! These mirror the primary blogger active-model / loader tests, but against fixtures whose
//! `belongs_to` fields keep the legacy `HasOne<E>` type (`common::blogger_legacy`), locking in
//! that the legacy path keeps working: nested writes set the foreign key, duplicate-target
//! relations are disambiguated, many-to-many junctions persist, and the entity loader hydrates
//! the relations.

mod common;

use crate::common::TestContext;
use crate::common::blogger_legacy::*;
use sea_orm::{Database, DbConn, DbErr, entity::*, prelude::*, query::*};
use tracing::info;

async fn setup(name: &str) -> TestContext {
    let ctx = TestContext::new(name).await;
    ctx.db
        .get_schema_builder()
        .register(user::Entity)
        .register(user_follower::Entity)
        .register(profile::Entity)
        .register(post::Entity)
        .register(post_tag::Entity)
        .register(tag::Entity)
        .register(attachment::Entity)
        .register(comment::Entity)
        .apply(&ctx.db)
        .await
        .unwrap();
    ctx
}

#[sea_orm_macros::test]
async fn test_legacy_belongs_to_nested_write() -> Result<(), DbErr> {
    let ctx = setup("test_legacy_belongs_to_nested_write").await;
    let db = &ctx.db;

    info!("save a user with a nested has_one profile");
    let alice = user::ActiveModel::builder()
        .set_name("Alice")
        .set_email("alice@sea-ql.org")
        .set_profile(profile::ActiveModel::builder().set_picture("alice.jpg"))
        .save(db)
        .await?;
    assert_eq!(alice.id, Unchanged(1));

    // the has_one child wrote its belongs_to foreign key back to this user
    let profile_row = profile::Entity::find().one(db).await?.expect("profile");
    assert_eq!(profile_row.user_id, 1);

    info!("save a post whose legacy belongs_to author is an existing user");
    let post = post::ActiveModel::builder()
        .set_title("post 1")
        .set_author(alice.clone())
        .save(db)
        .await?;
    // the belongs_to (HasOne) write set this row's own foreign key
    assert_eq!(post.user_id, alice.id);

    info!("save a post whose legacy belongs_to author is a brand-new nested user");
    let post2 = post::ActiveModel::builder()
        .set_title("post 2")
        .set_author(
            user::ActiveModel::builder()
                .set_name("Bob")
                .set_email("bob@sea-ql.org"),
        )
        .save(db)
        .await?;
    assert_eq!(post2.user_id, Unchanged(2));
    assert_eq!(user::Entity::find().count(db).await?, 2);

    ctx.delete().await;
    Ok(())
}

#[sea_orm_macros::test]
async fn test_legacy_many_to_many() -> Result<(), DbErr> {
    let ctx = setup("test_legacy_many_to_many").await;
    let db = &ctx.db;

    let author = user::ActiveModel::builder()
        .set_name("Alice")
        .set_email("alice@sea-ql.org")
        .save(db)
        .await?;

    info!("insert a post with two tags through the post_tag junction (legacy HasOne belongs_to)");
    let post = post::ActiveModel::builder()
        .set_title("A sunny day")
        .set_author(author)
        .add_tag(tag::ActiveModel::builder().set_tag("outdoor"))
        .add_tag(tag::ActiveModel::builder().set_tag("weather"))
        .save(db)
        .await?;

    let post_id = post.id.clone().unwrap();
    let loaded = post::Entity::load()
        .filter_by_id(post_id)
        .with(tag::Entity)
        .one(db)
        .await?
        .expect("post");
    let tags: Vec<String> = loaded.tags.iter().map(|t| t.tag.clone()).collect();
    assert_eq!(tags, ["outdoor", "weather"]);

    ctx.delete().await;
    Ok(())
}

#[sea_orm_macros::test]
async fn test_legacy_belongs_to_duplicate_target() -> Result<(), DbErr> {
    let ctx = setup("test_legacy_belongs_to_duplicate_target").await;
    let db = &ctx.db;

    // `user_follower` has two belongs_to fields — `user` and `follower` — both targeting
    // `user::Entity`, here declared with the legacy `HasOne<E>` type. Each nested write must
    // still set its own foreign key, disambiguated by relation (`follower` via its
    // `relation_enum`, `user` via the default inferred variant).
    let alice = user::ActiveModel::builder()
        .set_name("Alice")
        .set_email("alice@sea-ql.org")
        .save(db)
        .await?;
    let bob = user::ActiveModel::builder()
        .set_name("Bob")
        .set_email("bob@sea-ql.org")
        .save(db)
        .await?;

    let follow = user_follower::ActiveModelEx {
        user: ActiveHasOne::set(Some(alice)),
        follower: ActiveHasOne::set(Some(bob)),
        ..Default::default()
    }
    .insert(db)
    .await?;

    assert_eq!(follow.user_id, 1);
    assert_eq!(follow.follower_id, 2);

    let row = user_follower::Entity::find().one(db).await?.expect("row");
    assert_eq!((row.user_id, row.follower_id), (1, 2));

    ctx.delete().await;
    Ok(())
}

#[sea_orm_macros::test]
async fn test_legacy_entity_loader() -> Result<(), DbErr> {
    let ctx = setup("test_legacy_entity_loader").await;
    let db = &ctx.db;

    user::ActiveModel::builder()
        .set_name("Alice")
        .set_email("alice@sea-ql.org")
        .set_profile(profile::ActiveModel::builder().set_picture("alice.jpg"))
        .add_post(post::ActiveModel::builder().set_title("post 1"))
        .add_post(post::ActiveModel::builder().set_title("post 2"))
        .save(db)
        .await?;

    info!("load the user together with its has_one profile and has_many posts");
    let user = user::Entity::load()
        .filter_by_email("alice@sea-ql.org")
        .with(profile::Entity)
        .with(post::Entity)
        .one(db)
        .await?
        .expect("user");

    assert_eq!(user.profile.as_ref().expect("profile").picture, "alice.jpg");
    let titles: Vec<String> = user.posts.iter().map(|p| p.title.clone()).collect();
    assert_eq!(titles, ["post 1", "post 2"]);

    ctx.delete().await;
    Ok(())
}
