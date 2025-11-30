#![allow(unused_imports, dead_code)]

mod common;

use crate::common::TestContext;
use sea_orm::{Database, DbConn, DbErr, entity::*, prelude::*, query::*};
use tracing::info;

#[sea_orm_macros::test]
async fn test_active_model_ex_blog() -> Result<(), DbErr> {
    use common::blogger::*;

    let ctx = TestContext::new("test_active_model_ex_blog").await;
    let db = &ctx.db;

    db.get_schema_builder()
        .register(user::Entity)
        .register(user_follower::Entity)
        .register(profile::Entity)
        .register(post::Entity)
        .register(post_tag::Entity)
        .register(tag::Entity)
        .register(attachment::Entity)
        .register(comment::Entity)
        .apply(db)
        .await?;

    info!("save a new user");
    let user = user::ActiveModel::builder()
        .set_name("Alice")
        .set_email("@1")
        .save(db)
        .await?;

    assert_eq!(user.id, Unchanged(1));

    info!("save a post with an existing user");
    let post = post::ActiveModel::builder()
        .set_title("post 1")
        .set_author(user)
        .save(db)
        .await?;

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
                followers: HasManyModel::NotSet,
                following: HasManyModel::NotSet,
            }),
            comments: HasManyModel::NotSet,
            attachments: HasManyModel::NotSet,
            tags: HasManyModel::NotSet,
        }
    );

    info!("save a post with a new user");
    let post = post::ActiveModel::builder()
        .set_title("post 2")
        .set_author(user::ActiveModel::builder().set_name("Bob").set_email("@2"))
        .save(db)
        .await?;

    if false {
        post::ActiveModelEx {
            title: Set("post 2".into()),
            author: HasOneModel::set(user::ActiveModelEx {
                name: Set("Bob".into()),
                email: Set("@2".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
    }

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

    info!("save a new user with a new profile");
    let user = user::ActiveModel::builder()
        .set_name("Sam")
        .set_email("@3")
        .set_profile(profile::ActiveModel::builder().set_picture("Sam.jpg"))
        .save(db)
        .await?;

    if false {
        user::ActiveModelEx {
            name: Set("Sam".into()),
            email: Set("@3".into()),
            profile: HasOneModel::set(profile::ActiveModelEx {
                picture: Set("Sam.jpg".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
    }

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

    info!("save a new user with a new profile and 2 posts");
    let mut user = user::ActiveModel::builder()
        .set_name("Alan")
        .set_email("@4")
        .set_profile(profile::ActiveModel::builder().set_picture("Alan.jpg"))
        .add_post(post::ActiveModel::builder().set_title("post 3"))
        .add_post(post::ActiveModel::builder().set_title("post 4"))
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
            followers: HasManyModel::NotSet,
            following: HasManyModel::NotSet,
        }
    );

    let posts = user.find_related(post::Entity).all(db).await?;
    assert_eq!(posts.len(), 2);
    assert_eq!(posts[0].id, 3);
    assert_eq!(posts[1].id, 4);

    info!("replace posts of user: delete 3,4; insert 5 with attachment");
    user.posts = HasManyModel::Replace(vec![post::ActiveModelEx {
        title: Set("post 5".into()),
        attachments: HasManyModel::Append(vec![attachment::ActiveModelEx {
            file: Set("for post 5".into()),
            ..Default::default()
        }]),
        ..Default::default()
    }]);

    let mut user = user.save(db).await?;

    let posts = user.find_related(post::Entity).all(db).await?;
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].id, 5);
    let attachments = posts[0].find_related(attachment::Entity).all(db).await?;
    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0].id, 1);
    assert_eq!(attachments[0].file, "for post 5");

    info!("insert attachment for later use");
    let attachment_6 = attachment::ActiveModel::builder()
        .set_file("for post 6")
        .save(db)
        .await?;

    info!("add new post to user: insert 6 and attach existing attachment");
    user.posts = HasManyModel::Append(vec![post::ActiveModelEx {
        title: Set("post 6".into()),
        attachments: HasManyModel::Append(vec![attachment_6]),
        ..Default::default()
    }]);

    let mut user = user.save(db).await?;

    let posts = user.find_related(post::Entity).all(db).await?;
    assert_eq!(posts.len(), 2);
    assert_eq!(posts[0].id, 5);
    assert_eq!(posts[1].id, 6);
    let attachments = posts[1].find_related(attachment::Entity).all(db).await?;
    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0].file, "for post 6");

    info!("update post 6 through user");
    user.posts[0].title = Set("post 6!".into());

    let mut user = user.save(db).await?;

    let posts = user.find_related(post::Entity).all(db).await?;
    assert_eq!(posts.len(), 2);
    assert_eq!(posts[0].id, 5);
    assert_eq!(posts[0].title, "post 5");
    assert_eq!(posts[1].id, 6);
    assert_eq!(posts[1].title, "post 6!");

    info!("update user profile and delete all posts");
    user.profile.as_mut().unwrap().picture = Set("Alan2.jpg".into());
    // user.posts = HasManyModel::Replace(vec![]);
    user.posts.replace_all([]);
    user.save(db).await?;

    info!("check that user has 0 posts");
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
            followers: HasMany::Unloaded,
            following: HasMany::Unloaded,
        }
    );

    info!("check that attachment still exists");
    let attachment_1 = attachment::Entity::find_by_id(1).one(db).await?.unwrap();
    assert_eq!(attachment_1.file, "for post 5");
    assert!(attachment_1.post_id.is_none());

    info!("insert one tag for later use");
    let day = tag::ActiveModel {
        tag: Set("day".into()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    info!("insert new post and set 2 tags");
    let post_ = post::ActiveModelEx {
        id: NotSet,
        user_id: NotSet,
        title: Set("post 7".into()),
        author: HasOneModel::set(user.clone().into_active_model()),
        comments: HasManyModel::NotSet,
        attachments: HasManyModel::NotSet,
        tags: HasManyModel::Append(vec![
            day.clone().into_active_model().into(),
            tag::ActiveModel {
                id: NotSet,
                tag: Set("pet".into()),
            }
            .into(),
        ]),
    };

    let post = post::ActiveModel::builder()
        .set_title("post 7")
        .set_author(user.into_active_model())
        .add_tag(day.into_active_model())
        .add_tag(tag::ActiveModel::builder().set_tag("pet"));

    assert_eq!(post, post_);

    let post = post.save(db).await?;

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
                followers: HasManyModel::NotSet,
                following: HasManyModel::NotSet,
            }),
            comments: HasManyModel::NotSet,
            attachments: HasManyModel::NotSet,
            tags: HasManyModel::Append(vec![
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

    info!("replace should be idempotent");
    let mut post = post.save(db).await?;

    info!("append should be idempotent");
    post.tags.convert_to_append();
    let mut post = post.save(db).await?;

    info!("get back the post and tags");
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

    info!("add attachment to post");
    let mut post_7 = post_7.into_active_model();
    post_7.attachments.push(attachment::ActiveModel {
        file: Set("for post 7".into()),
        ..Default::default()
    });
    post_7.insert(db).await?;

    info!("get back the post and attachment");
    let post_7 = post::Entity::load()
        .filter_by_id(7)
        .with(attachment::Entity)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(post_7.attachments.len(), 1);
    assert_eq!(post_7.attachments[0].file, "for post 7");

    info!("update user profile through post");
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

    info!("replace post tags: remove tag 1 add tag 3");
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

    info!("get back the post and tags");
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

    info!("update post title and add new tag");
    post.title = Set("post 7!".into());
    post.tags = HasManyModel::Append(vec![
        tag::ActiveModel {
            id: NotSet, // new tag
            tag: Set("sunny".into()),
        }
        .into(),
    ]);
    post.save(db).await?;

    info!("get back the post and tags");
    let post_7 = post::Entity::load()
        .filter_by_id(7)
        .with(tag::Entity)
        .one(db)
        .await?
        .unwrap();

    assert_eq!(post_7.id, 7);
    assert_eq!(post_7.title, "post 7!");
    assert_eq!(post_7.tags.len(), 3);
    assert_eq!(post_7.tags[0].tag, "pet");
    assert_eq!(post_7.tags[1].tag, "food");
    assert_eq!(post_7.tags[2].tag, "sunny");

    let user_1 = user::Entity::find_by_email("@1").one(db).await?.unwrap();
    info!("can't delete as there are posts belonging to user");
    assert!(user_1.clone().delete(db).await.is_err());
    info!("cascade delete user 1");
    assert_eq!(user_1.cascade_delete(db).await?.rows_affected, 2); // user + post
    assert!(user::Entity::find_by_email("@1").one(db).await?.is_none());

    info!("cascade delete user 2");
    let user_2 = user::Entity::find_by_email("@2").one(db).await?.unwrap();
    assert_eq!(user_2.cascade_delete(db).await?.rows_affected, 2); // user + post
    assert!(user::Entity::find_by_email("@2").one(db).await?.is_none());

    info!("cascade delete user 4");
    let user_4 = user::Entity::find_by_id(4).one(db).await?.unwrap();
    assert_eq!(
        user_4.cascade_delete(db).await?.rows_affected,
        1 + 1 + 3 + 1
    ); // user + profile + post_tag + post
    assert!(user::Entity::find_by_id(4).one(db).await?.is_none());

    info!("insert a new user with a new profile and new post with tag");
    let user = user::ActiveModel::builder()
        .set_name("Bob")
        .set_email("bob@sea-ql.org")
        .set_profile(profile::ActiveModel::builder().set_picture("image.jpg"))
        .add_post(
            post::ActiveModel::builder()
                .set_title("Nice weather")
                .add_tag(tag::ActiveModel::builder().set_tag("sunny")),
        )
        .insert(db)
        .await?;

    info!("get back the user with profile, posts and tags");
    assert_eq!(
        user::Entity::load()
            .filter_by_id(user.id)
            .with(profile::Entity)
            .with((post::Entity, tag::Entity))
            .one(db)
            .await?
            .unwrap(),
        user::ModelEx {
            id: 5,
            name: "Bob".into(),
            email: "bob@sea-ql.org".into(),
            profile: HasOne::loaded(profile::Model {
                id: 3,
                picture: "image.jpg".into(),
                user_id: 5,
            }),
            posts: HasMany::Loaded(vec![post::ModelEx {
                id: 8,
                user_id: 5,
                title: "Nice weather".into(),
                author: HasOne::Unloaded,
                attachments: HasMany::Unloaded,
                comments: HasMany::Unloaded,
                tags: HasMany::Loaded(vec![tag::ModelEx {
                    id: 5,
                    tag: "sunny".into(),
                    posts: HasMany::Unloaded,
                }]),
            }]),
            followers: HasMany::Unloaded,
            following: HasMany::Unloaded,
        }
    );

    info!("should be no-op");
    assert_eq!(user, user.clone().into_active_model().update(db).await?);

    // test self_ref via

    info!("save a new user Alice");
    let alice = user::ActiveModel::builder()
        .set_name("Alice")
        .set_email("@alice")
        .save(db)
        .await?;

    let bob = user::Entity::find()
        .filter(user::COLUMN.name.eq("Bob"))
        .one(db)
        .await?
        .unwrap()
        .into_active_model()
        .into_ex();

    let sam = user::Entity::find()
        .filter(user::COLUMN.name.eq("Sam"))
        .one(db)
        .await?
        .unwrap()
        .into_active_model();

    info!("Add follower to Alice");
    let alice = alice.add_follower(bob.clone()).save(db).await?;

    info!("Sam starts following Alice");
    sam.clone().into_ex().add_following(alice).save(db).await?;

    info!("Add follower to Bob");
    bob.add_follower(sam).save(db).await?;

    let users = user::Entity::load()
        .with(user_follower::Entity)
        .with(user_follower::Entity::REVERSE)
        .order_by_asc(user::COLUMN.name)
        .all(db)
        .await?;

    assert_eq!(users[0].name, "Alice");
    assert_eq!(users[0].followers.len(), 2);
    assert_eq!(users[0].followers[0].name, "Sam");
    assert_eq!(users[0].followers[1].name, "Bob");
    assert!(users[0].following.is_empty());

    assert_eq!(users[1].name, "Bob");
    assert_eq!(users[1].followers.len(), 1);
    assert_eq!(users[1].followers[0].name, "Sam");
    assert_eq!(users[1].following.len(), 1);
    assert_eq!(users[1].following[0].name, "Alice");

    assert_eq!(users[2].name, "Sam");
    assert!(users[2].followers.is_empty());
    assert_eq!(users[2].following.len(), 2);
    assert_eq!(users[2].following[0].name, "Bob");
    assert_eq!(users[2].following[1].name, "Alice");

    Ok(())
}

#[sea_orm_macros::test]
async fn test_active_model_ex_film_store() -> Result<(), DbErr> {
    use common::film_store::*;

    let ctx = TestContext::new("test_active_model_ex_film_store").await;
    let db = &ctx.db;

    db.get_schema_builder()
        .register(film::Entity)
        .register(actor::Entity)
        .register(film_actor::Entity)
        .register(staff::Entity)
        .apply(db)
        .await?;

    info!("save film Mission, no actors");
    let mut film = film::ActiveModel {
        title: Set("Mission".into()),
        ..Default::default()
    }
    .save(db)
    .await?
    .into_ex();

    info!("create two actors and add to film Mission");
    film.actors.push(actor::ActiveModel {
        name: Set("Tom".into()),
        ..Default::default()
    });
    film.actors.push(actor::ActiveModel {
        name: Set("Ben".into()),
        ..Default::default()
    });
    film.save(db).await?;

    info!("check that film has two actors");
    let film = film::Entity::load()
        .with(actor::Entity)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(film.title, "Mission");
    assert_eq!(film.actors.len(), 2);
    assert_eq!(film.actors[0].name, "Tom");
    assert_eq!(film.actors[1].name, "Ben");

    info!("save new actor Sam, no films");
    let tom = film.actors.into_iter().next().unwrap();
    let sam = actor::ActiveModel {
        // new actor
        name: Set("Sam".into()),
        ..Default::default()
    }
    .save(db)
    .await?;

    info!("save new films Galaxy with Tom and Sam as actors");
    film::ActiveModelEx {
        title: Set("Galaxy".into()),
        actors: HasManyModel::Replace(vec![tom.into_active_model(), sam.into_ex()]),
        ..Default::default()
    }
    .save(db)
    .await?;

    info!("film Galaxy has two actors");
    let film = film::Entity::load()
        .filter_by_id(2)
        .with(actor::Entity)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(film.title, "Galaxy");
    assert_eq!(film.actors.len(), 2);
    assert_eq!(film.actors[0].name, "Tom");
    assert_eq!(film.actors[1].name, "Sam");

    info!("actor Tom has two films");
    let tom = actor::Entity::load()
        .filter_by_name("Tom")
        .with(film::Entity)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(tom.name, "Tom");
    assert_eq!(tom.films.len(), 2);
    assert_eq!(tom.films[0].title, "Mission");
    assert_eq!(tom.films[1].title, "Galaxy");

    info!("cascade delete film Galaxy");
    assert_eq!(film.delete(db).await?.rows_affected, 3); // film + 2 film_actor

    info!("tom has 1 film left");
    let tom = actor::Entity::load()
        .filter_by_name("Tom")
        .with(film::Entity)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(tom.name, "Tom");
    assert_eq!(tom.films.len(), 1);
    assert_eq!(tom.films[0].title, "Mission");

    info!("sam still exists, but no films");
    let sam = actor::Entity::load()
        .filter_by_name("Sam")
        .with(film::Entity)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(sam.name, "Sam");
    assert_eq!(sam.films.len(), 0);

    info!("should be idempotent");
    let mut film = film::Entity::find_by_id(1)
        .one(db)
        .await?
        .unwrap()
        .into_active_model()
        .into_ex();
    film.actors.push(tom.into_active_model());
    film.save(db).await?;

    // test self_ref

    info!("insert new staff: alan");

    let alan = staff::ActiveModel::builder()
        .set_name("Alan")
        .insert(db)
        .await?;

    info!("insert new staff: Ben reports to Alan");
    staff::ActiveModel::builder()
        .set_name("Ben")
        .set_reports_to(alan.clone())
        .insert(db)
        .await?;

    info!("insert new staff: Alice");
    let alice = staff::ActiveModel::builder()
        .set_name("Alice")
        .insert(db)
        .await?;

    info!("assign Alice to report to Alan");
    let alan = alan
        .into_active_model()
        .add_manage(alice.clone())
        .save(db)
        .await?;

    info!("insert new staff: Elle");
    staff::ActiveModel::builder()
        .set_name("Elle")
        .insert(db)
        .await?;

    info!("load all staff");
    let staff = staff::Entity::load()
        .with(staff::Relation::ReportsTo)
        .with(staff::Relation::Manages)
        .all(db)
        .await?;

    assert_eq!(staff[0].name, "Alan");
    assert_eq!(staff[0].reports_to, None);
    assert_eq!(staff[0].manages[0].name, "Ben");
    assert_eq!(staff[0].manages[1].name, "Alice");

    assert_eq!(staff[1].name, "Ben");
    assert_eq!(staff[1].reports_to.as_ref().unwrap().name, "Alan");
    assert!(staff[1].manages.is_empty());

    assert_eq!(staff[2].name, "Alice");
    assert_eq!(staff[1].reports_to.as_ref().unwrap().name, "Alan");
    assert!(staff[2].manages.is_empty());

    assert_eq!(staff[3].name, "Elle");
    assert_eq!(staff[3].reports_to, None);
    assert!(staff[3].manages.is_empty());

    info!("delete alan, reports_to should be cleared");
    alan.delete(db).await?;

    info!("verify Alice still exists");
    assert!(
        staff::Entity::find_by_id(alice.id)
            .one(db)
            .await?
            .unwrap()
            .reports_to_id
            .is_none()
    );

    Ok(())
}
