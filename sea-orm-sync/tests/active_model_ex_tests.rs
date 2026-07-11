#![allow(unused_imports, dead_code)]

mod common;

use crate::common::TestContext;
use sea_orm::{Database, DbConn, DbErr, entity::*, prelude::*, query::*};
use tracing::info;

mod optional_self_ref {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "optional_self_ref")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(enum_name = "ParentId")]
        pub parent_ref: Option<i32>,
        #[sea_orm(self_ref, relation_enum = "Parent", from = "ParentId", to = "id")]
        pub parent: BelongsTo<Option<Entity>>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod mixed_composite_parent {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "mixed_composite_parent")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id1: i32,
        #[sea_orm(primary_key, auto_increment = false)]
        pub id2: i32,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod mixed_composite_child {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "mixed_composite_child")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub parent_id1: i32,
        pub parent_id2: Option<i32>,
        #[sea_orm(belongs_to, from = "(parent_id1, parent_id2)", to = "(id1, id2)")]
        pub parent: BelongsTo<Option<super::mixed_composite_parent::Entity>>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

#[sea_orm_macros::test]
fn test_active_model_ex_blog() -> Result<(), DbErr> {
    use common::blogger::*;

    let ctx = TestContext::new("test_active_model_ex_blog");
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
        .apply(db)?;

    info!("save a new user");
    let user = user::ActiveModel::builder()
        .set_name("Alice")
        .set_email("@1")
        .save(db)?;

    assert_eq!(user.id, Unchanged(1));

    info!("save a post with an existing user");
    let post = post::ActiveModel::builder()
        .set_title("post 1")
        .set_author(user)
        .save(db)?;

    assert_eq!(
        post,
        post::ActiveModelEx {
            id: Unchanged(1),
            user_id: Unchanged(1),
            title: Unchanged("post 1".into()),
            author: ActiveBelongsTo::set(user::ActiveModelEx {
                id: Unchanged(1),
                name: Unchanged("Alice".into()),
                email: Unchanged("@1".into()),
                profile: ActiveHasOne::NotSet,
                posts: ActiveHasMany::NotSet,
                followers: ActiveHasMany::NotSet,
                following: ActiveHasMany::NotSet,
            }),
            comments: ActiveHasMany::NotSet,
            attachments: ActiveHasMany::NotSet,
            tags: ActiveHasMany::NotSet,
        }
    );

    info!("save a post with a new user");
    let post = post::ActiveModel::builder()
        .set_title("post 2")
        .set_author(user::ActiveModel::builder().set_name("Bob").set_email("@2"))
        .save(db)?;

    if false {
        post::ActiveModelEx {
            title: Set("post 2".into()),
            author: ActiveBelongsTo::set(user::ActiveModelEx {
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
            author: ActiveBelongsTo::set(user::ActiveModelEx {
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
        .save(db)?;

    if false {
        user::ActiveModelEx {
            name: Set("Sam".into()),
            email: Set("@3".into()),
            profile: ActiveHasOne::set(Some(profile::ActiveModelEx {
                picture: Set("Sam.jpg".into()),
                ..Default::default()
            })),
            ..Default::default()
        };
    }

    assert_eq!(
        user,
        user::ActiveModelEx {
            id: Unchanged(3),
            name: Unchanged("Sam".into()),
            email: Unchanged("@3".into()),
            profile: ActiveHasOne::set(Some(profile::ActiveModelEx {
                id: Unchanged(1),
                picture: Unchanged("Sam.jpg".into()),
                user_id: Unchanged(3),
                user: ActiveBelongsTo::NotSet,
            })),
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
        .save(db)?;

    assert_eq!(
        user,
        user::ActiveModelEx {
            id: Unchanged(4),
            name: Unchanged("Alan".into()),
            email: Unchanged("@4".into()),
            profile: ActiveHasOne::set(Some(profile::ActiveModelEx {
                id: Unchanged(2),
                picture: Unchanged("Alan.jpg".into()),
                user_id: Unchanged(4),
                user: ActiveBelongsTo::NotSet,
            })),
            posts: ActiveHasMany::Append(vec![
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
            followers: ActiveHasMany::NotSet,
            following: ActiveHasMany::NotSet,
        }
    );

    let posts = user.find_related(post::Entity).all(db)?;
    assert_eq!(posts.len(), 2);
    assert_eq!(posts[0].id, 3);
    assert_eq!(posts[1].id, 4);

    info!("replace posts of user: delete 3,4; insert 5 with attachment");
    user.posts = ActiveHasMany::Replace(vec![post::ActiveModelEx {
        title: Set("post 5".into()),
        attachments: ActiveHasMany::Append(vec![attachment::ActiveModelEx {
            file: Set("for post 5".into()),
            ..Default::default()
        }]),
        ..Default::default()
    }]);

    let mut user = user.save(db)?;

    let posts = user.find_related(post::Entity).all(db)?;
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].id, 5);
    let attachments = posts[0].find_related(attachment::Entity).all(db)?;
    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0].id, 1);
    assert_eq!(attachments[0].file, "for post 5");

    info!("insert attachment for later use");
    let attachment_6 = attachment::ActiveModel::builder()
        .set_file("for post 6")
        .save(db)?;

    info!("add new post to user: insert 6 and attach existing attachment");
    user.posts = ActiveHasMany::Append(vec![post::ActiveModelEx {
        title: Set("post 6".into()),
        attachments: ActiveHasMany::Append(vec![attachment_6]),
        ..Default::default()
    }]);

    let mut user = user.save(db)?;

    let posts = user.find_related(post::Entity).all(db)?;
    assert_eq!(posts.len(), 2);
    assert_eq!(posts[0].id, 5);
    assert_eq!(posts[1].id, 6);
    let attachments = posts[1].find_related(attachment::Entity).all(db)?;
    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0].file, "for post 6");

    info!("update post 6 through user");
    user.posts[0].title = Set("post 6!".into());

    let mut user = user.save(db)?;

    let posts = user.find_related(post::Entity).all(db)?;
    assert_eq!(posts.len(), 2);
    assert_eq!(posts[0].id, 5);
    assert_eq!(posts[0].title, "post 5");
    assert_eq!(posts[1].id, 6);
    assert_eq!(posts[1].title, "post 6!");

    info!("update user profile and delete all posts");
    user.profile.as_mut().unwrap().picture = Set("Alan2.jpg".into());
    // user.posts = ActiveHasMany::Replace(vec![]);
    user.posts.replace_all([]);
    user.save(db)?;

    info!("check that user has 0 posts");
    let user = user::Entity::load()
        .filter_by_id(4)
        .with(profile::Entity)
        .with(post::Entity)
        .one(db)?
        .unwrap();

    assert_eq!(
        user,
        user::ModelEx {
            id: 4,
            name: "Alan".into(),
            email: "@4".into(),
            profile: HasOne::loaded(Some(profile::Model {
                id: 2,
                picture: "Alan2.jpg".into(),
                user_id: 4,
            })),
            posts: HasMany::Loaded(vec![]),
            followers: HasMany::Unloaded,
            following: HasMany::Unloaded,
        }
    );

    info!("check that attachment still exists");
    let attachment_1 = attachment::Entity::find_by_id(1).one(db)?.unwrap();
    assert_eq!(attachment_1.file, "for post 5");
    assert!(attachment_1.post_id.is_none());

    info!("insert one tag for later use");
    let day = tag::ActiveModel {
        tag: Set("day".into()),
        ..Default::default()
    }
    .insert(db)?;

    info!("insert new post and set 2 tags");
    let post_ = post::ActiveModelEx {
        id: NotSet,
        user_id: NotSet,
        title: Set("post 7".into()),
        author: ActiveBelongsTo::set(user.clone().into_active_model()),
        comments: ActiveHasMany::NotSet,
        attachments: ActiveHasMany::NotSet,
        tags: ActiveHasMany::Append(vec![
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

    let post = post.save(db)?;

    assert_eq!(
        post,
        post::ActiveModelEx {
            id: Unchanged(7),
            user_id: Unchanged(4),
            title: Unchanged("post 7".into()),
            author: ActiveBelongsTo::set(user::ActiveModelEx {
                id: Unchanged(4),
                name: Unchanged("Alan".into()),
                email: Unchanged("@4".into()),
                profile: ActiveHasOne::set(Some(profile::ActiveModelEx {
                    id: Unchanged(2),
                    picture: Unchanged("Alan2.jpg".into()),
                    user_id: Unchanged(4),
                    user: ActiveBelongsTo::NotSet,
                },)),
                posts: ActiveHasMany::Append(vec![]),
                followers: ActiveHasMany::NotSet,
                following: ActiveHasMany::NotSet,
            }),
            comments: ActiveHasMany::NotSet,
            attachments: ActiveHasMany::NotSet,
            tags: ActiveHasMany::Append(vec![
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
    let mut post = post.save(db)?;

    info!("append should be idempotent");
    post.tags.convert_to_append();
    let mut post = post.save(db)?;

    info!("get back the post and tags");
    let post_7 = post::Entity::load()
        .filter_by_id(7)
        .with(tag::Entity)
        .one(db)?
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
    post_7.insert(db)?;

    info!("get back the post and attachment");
    let post_7 = post::Entity::load()
        .filter_by_id(7)
        .with(attachment::Entity)
        .one(db)?
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
    let mut post = post.save(db)?;
    assert_eq!(
        profile::Entity::find_by_id(2).one(db)?.unwrap().picture,
        "Alan3.jpg"
    );

    info!("replace post tags: remove tag 1 add tag 3");
    post.tags = ActiveHasMany::Replace(vec![
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
    let mut post = post.save(db)?;

    info!("get back the post and tags");
    let post_7 = post::Entity::load()
        .filter_by_id(7)
        .with(tag::Entity)
        .one(db)?
        .unwrap();

    assert_eq!(post_7.id, 7);
    assert_eq!(post_7.tags.len(), 2);
    assert_eq!(post_7.tags[0].tag, "pet");
    assert_eq!(post_7.tags[1].tag, "food");

    info!("update post title and add new tag");
    post.title = Set("post 7!".into());
    post.tags = ActiveHasMany::Append(vec![
        tag::ActiveModel {
            id: NotSet, // new tag
            tag: Set("sunny".into()),
        }
        .into(),
    ]);
    post.save(db)?;

    info!("get back the post and tags");
    let post_7 = post::Entity::load()
        .filter_by_id(7)
        .with(tag::Entity)
        .one(db)?
        .unwrap();

    assert_eq!(post_7.id, 7);
    assert_eq!(post_7.title, "post 7!");
    assert_eq!(post_7.tags.len(), 3);
    assert_eq!(post_7.tags[0].tag, "pet");
    assert_eq!(post_7.tags[1].tag, "food");
    assert_eq!(post_7.tags[2].tag, "sunny");

    let user_1 = user::Entity::find_by_email("@1").one(db)?.unwrap();
    info!("can't delete as there are posts belonging to user");
    assert!(user_1.clone().delete(db).is_err());
    info!("cascade delete user 1");
    assert_eq!(user_1.cascade_delete(db)?.rows_affected, 2); // user + post
    assert!(user::Entity::find_by_email("@1").one(db)?.is_none());

    info!("cascade delete user 2");
    let user_2 = user::Entity::find_by_email("@2").one(db)?.unwrap();
    assert_eq!(user_2.cascade_delete(db)?.rows_affected, 2); // user + post
    assert!(user::Entity::find_by_email("@2").one(db)?.is_none());

    info!("cascade delete user 4");
    let user_4 = user::Entity::find_by_id(4).one(db)?.unwrap();
    assert_eq!(user_4.cascade_delete(db)?.rows_affected, 1 + 1 + 3 + 1); // user + profile + post_tag + post
    assert!(user::Entity::find_by_id(4).one(db)?.is_none());

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
        .insert(db)?;

    info!("get back the user with profile, posts and tags");
    assert_eq!(
        user::Entity::load()
            .filter_by_id(user.id)
            .with(profile::Entity)
            .with((post::Entity, tag::Entity))
            .one(db)?
            .unwrap(),
        user::ModelEx {
            id: 5,
            name: "Bob".into(),
            email: "bob@sea-ql.org".into(),
            profile: HasOne::loaded(Some(profile::Model {
                id: 3,
                picture: "image.jpg".into(),
                user_id: 5,
            })),
            posts: HasMany::Loaded(vec![post::ModelEx {
                id: 8,
                user_id: 5,
                title: "Nice weather".into(),
                author: BelongsTo::Unloaded,
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
    assert_eq!(user, user.clone().into_active_model().update(db)?);

    // test self_ref via

    info!("save a new user Alice");
    let alice = user::ActiveModel::builder()
        .set_name("Alice")
        .set_email("@alice")
        .save(db)?;

    let bob = user::Entity::find()
        .filter(user::COLUMN.name.eq("Bob"))
        .one(db)?
        .unwrap()
        .into_active_model()
        .into_ex();

    let sam = user::Entity::find()
        .filter(user::COLUMN.name.eq("Sam"))
        .one(db)?
        .unwrap()
        .into_active_model();

    info!("Add follower to Alice");
    let alice = alice.add_follower(bob.clone()).save(db)?;

    info!("Sam starts following Alice");
    sam.clone().into_ex().add_following(alice).save(db)?;

    info!("Add follower to Bob");
    bob.add_follower(sam).save(db)?;

    let users = user::Entity::load()
        .with(user_follower::Entity)
        .with(user_follower::Entity::REVERSE)
        .order_by_asc(user::COLUMN.name)
        .all(db)?;

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
fn test_active_model_ex_film_store() -> Result<(), DbErr> {
    use common::film_store::*;

    let ctx = TestContext::new("test_active_model_ex_film_store");
    let db = &ctx.db;

    db.get_schema_builder()
        .register(film::Entity)
        .register(actor::Entity)
        .register(film_actor::Entity)
        .register(staff::Entity)
        .apply(db)?;

    info!("save film Mission, no actors");
    let mut film = film::ActiveModel {
        title: Set("Mission".into()),
        ..Default::default()
    }
    .save(db)?
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
    film.save(db)?;

    info!("check that film has two actors");
    let film = film::Entity::load().with(actor::Entity).one(db)?.unwrap();
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
    .save(db)?;

    info!("save new films Galaxy with Tom and Sam as actors");
    film::ActiveModelEx {
        title: Set("Galaxy".into()),
        actors: ActiveHasMany::Replace(vec![tom.into_active_model(), sam.into_ex()]),
        ..Default::default()
    }
    .save(db)?;

    info!("film Galaxy has two actors");
    let film = film::Entity::load()
        .filter_by_id(2)
        .with(actor::Entity)
        .one(db)?
        .unwrap();
    assert_eq!(film.title, "Galaxy");
    assert_eq!(film.actors.len(), 2);
    assert_eq!(film.actors[0].name, "Tom");
    assert_eq!(film.actors[1].name, "Sam");

    info!("actor Tom has two films");
    let tom = actor::Entity::load()
        .filter_by_name("Tom")
        .with(film::Entity)
        .one(db)?
        .unwrap();
    assert_eq!(tom.name, "Tom");
    assert_eq!(tom.films.len(), 2);
    assert_eq!(tom.films[0].title, "Mission");
    assert_eq!(tom.films[1].title, "Galaxy");

    info!("cascade delete film Galaxy");
    assert_eq!(film.delete(db)?.rows_affected, 3); // film + 2 film_actor

    info!("tom has 1 film left");
    let tom = actor::Entity::load()
        .filter_by_name("Tom")
        .with(film::Entity)
        .one(db)?
        .unwrap();
    assert_eq!(tom.name, "Tom");
    assert_eq!(tom.films.len(), 1);
    assert_eq!(tom.films[0].title, "Mission");

    info!("sam still exists, but no films");
    let sam = actor::Entity::load()
        .filter_by_name("Sam")
        .with(film::Entity)
        .one(db)?
        .unwrap();
    assert_eq!(sam.name, "Sam");
    assert_eq!(sam.films.len(), 0);

    info!("should be idempotent");
    let mut film = film::Entity::find_by_id(1)
        .one(db)?
        .unwrap()
        .into_active_model()
        .into_ex();
    film.actors.push(tom.into_active_model());
    film.save(db)?;

    // test self_ref

    info!("insert new staff: alan");

    let alan = staff::ActiveModel::builder().set_name("Alan").insert(db)?;

    info!("insert new staff: Ben reports to Alan");
    staff::ActiveModel::builder()
        .set_name("Ben")
        .set_reports_to(alan.clone())
        .insert(db)?;

    info!("insert new staff: Alice");
    let alice = staff::ActiveModel::builder().set_name("Alice").insert(db)?;

    info!("assign Alice to report to Alan");
    let alan = alan
        .into_active_model()
        .add_manage(alice.clone())
        .save(db)?;

    info!("insert new staff: Elle");
    staff::ActiveModel::builder().set_name("Elle").insert(db)?;

    info!("load all staff");
    let staff = staff::Entity::load()
        .with(staff::Relation::ReportsTo)
        .with(staff::Relation::Manages)
        .all(db)?;

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
    alan.delete(db)?;

    info!("verify Alice still exists");
    assert!(
        staff::Entity::find_by_id(alice.id)
            .one(db)?
            .unwrap()
            .reports_to_id
            .is_none()
    );

    Ok(())
}

#[sea_orm_macros::test]
fn test_has_one_replace_and_delete() -> Result<(), DbErr> {
    use common::blogger::*;

    let ctx = TestContext::new("test_has_one_replace_and_delete");
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
        .apply(db)?;

    info!("#3061: replacing a populated HasOne deletes the old record instead of erroring");
    let user = user::ActiveModel::builder()
        .set_name("Rick")
        .set_email("rick@sea-ql.org")
        .set_profile(profile::ActiveModel::builder().set_picture("first.jpg"))
        .save(db)?;

    let user = user
        .set_profile(profile::ActiveModel::builder().set_picture("second.jpg"))
        .save(db)?;

    let profiles = profile::Entity::find().all(db)?;
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].picture, "second.jpg");

    info!("#3060: clear the HasOne via the generated clear_<field> builder");
    user.clear_profile().save(db)?;

    assert!(profile::Entity::find().all(db)?.is_empty());

    ctx.delete();

    Ok(())
}

#[sea_orm_macros::test]
fn test_belongs_to_duplicate_target() -> Result<(), DbErr> {
    use common::blogger::*;

    let ctx = TestContext::new("test_belongs_to_duplicate_target");
    let db = &ctx.db;

    db.get_schema_builder()
        .register(user::Entity)
        .register(user_follower::Entity)
        .apply(db)?;

    // `user_follower` has two belongs_to fields — `user` and `follower` — both
    // targeting `user::Entity`. Nested writes on such duplicate-target relations
    // used to be silently skipped; now each writes its own FK, disambiguated by
    // relation (`follower` via its `relation_enum`, `user` via the default).
    let alice = user::ActiveModel::builder()
        .set_name("Alice")
        .set_email("alice@sea-ql.org")
        .save(db)?;
    let bob = user::ActiveModel::builder()
        .set_name("Bob")
        .set_email("bob@sea-ql.org")
        .save(db)?;

    info!("link the two users through the disambiguated nested belongs_to");
    let follow = user_follower::ActiveModelEx {
        user: ActiveBelongsTo::set(alice),
        follower: ActiveBelongsTo::set(bob),
        ..Default::default()
    }
    .insert(db)?;

    // Each belongs_to wrote its own FK (previously a silent no-op).
    assert_eq!(follow.user_id, 1);
    assert_eq!(follow.follower_id, 2);

    let row = user_follower::Entity::find().one(db)?.expect("row");
    assert_eq!(row.user_id, 1);
    assert_eq!(row.follower_id, 2);

    ctx.delete();

    Ok(())
}

#[sea_orm_macros::test]
fn test_clear_belongs_to_clears_unset_fk() -> Result<(), DbErr> {
    use common::bakery_dense::{bakery, cake};

    let ctx = TestContext::new("test_clear_belongs_to_clears_unset_fk");
    let db = &ctx.db;

    db.get_schema_builder()
        .register(bakery::Entity)
        .register(cake::Entity)
        .apply(db)?;

    let bakery = bakery::ActiveModel::builder()
        .set_name("Sea")
        .set_profit_margin(0.0)
        .insert(db)?;
    let bakery_id = bakery.id;
    let cake = cake::ActiveModel::builder()
        .set_name("Plain")
        .set_price(Decimal::from(5))
        .set_gluten_free(true)
        .set_serial(Uuid::nil())
        .set_bakery(bakery.clone())
        .insert(db)?;
    let cake_with_option = cake::ActiveModel::builder()
        .set_name("Option")
        .set_price(Decimal::from(6))
        .set_gluten_free(true)
        .set_serial(Uuid::nil())
        .set_bakery(bakery.clone())
        .insert(db)?;

    assert_eq!(cake.bakery_id, Some(bakery_id));
    assert_eq!(cake_with_option.bakery_id, Some(bakery_id));

    let partial_cake = |id| cake::ActiveModelEx {
        id: Unchanged(id),
        name: NotSet,
        price: NotSet,
        bakery_id: NotSet,
        gluten_free: NotSet,
        serial: NotSet,
        bakery: ActiveBelongsTo::NotSet,
        lineitems: ActiveHasMany::NotSet,
        bakers: ActiveHasMany::NotSet,
    };

    let cleared = partial_cake(cake.id).clear_bakery().update(db)?;

    assert!(cleared.bakery_id.is_none());
    let row = cake::Entity::find_by_id(cake.id).one(db)?.expect("cake");
    assert!(row.bakery_id.is_none());

    let cleared = partial_cake(cake_with_option.id)
        .set_bakery_option(None::<bakery::ActiveModelEx>)
        .update(db)?;

    assert!(cleared.bakery_id.is_none());
    let row = cake::Entity::find_by_id(cake_with_option.id)
        .one(db)?
        .expect("cake");
    assert!(row.bakery_id.is_none());

    ctx.delete();

    Ok(())
}

#[sea_orm_macros::test]
fn test_clear_self_ref_belongs_to_clears_unset_fk() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_clear_self_ref_belongs_to_clears_unset_fk");
    let db = &ctx.db;

    db.get_schema_builder()
        .register(optional_self_ref::Entity)
        .apply(db)?;

    let parent = optional_self_ref::ActiveModel::builder().insert(db)?;
    let parent_id = parent.id;
    let child = optional_self_ref::ActiveModel::builder()
        .set_parent(parent)
        .insert(db)?;

    assert_eq!(child.parent_ref, Some(parent_id));

    let cleared = optional_self_ref::ActiveModelEx {
        id: Unchanged(child.id),
        parent_ref: NotSet,
        parent: ActiveBelongsTo::NotSet,
    }
    .clear_parent()
    .update(db)?;

    assert!(cleared.parent_ref.is_none());
    let row = optional_self_ref::Entity::find_by_id(child.id)
        .one(db)?
        .expect("child");
    assert!(row.parent_ref.is_none());

    ctx.delete();

    Ok(())
}

#[sea_orm_macros::test]
fn test_clear_mixed_nullable_composite_belongs_to() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_clear_mixed_nullable_composite_belongs_to");
    let db = &ctx.db;

    db.get_schema_builder()
        .register(mixed_composite_parent::Entity)
        .register(mixed_composite_child::Entity)
        .apply(db)?;

    mixed_composite_parent::ActiveModel::builder()
        .set_id1(1)
        .set_id2(2)
        .insert(db)?;
    let child = mixed_composite_child::ActiveModel::builder()
        .set_parent_id1(1)
        .set_parent_id2(Some(2))
        .insert(db)?;

    let cleared = mixed_composite_child::ActiveModelEx {
        id: Unchanged(child.id),
        parent_id1: NotSet,
        parent_id2: NotSet,
        parent: ActiveBelongsTo::NotSet,
    }
    .clear_parent()
    .update(db)?;

    assert_eq!(cleared.parent_id1, 1);
    assert_eq!(cleared.parent_id2, None);

    ctx.delete();

    Ok(())
}
