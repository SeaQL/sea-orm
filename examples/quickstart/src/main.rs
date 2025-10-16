use sea_orm::{Database, DbErr};

mod user {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "user")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        #[sea_orm(unique)]
        pub email: String,
        #[sea_orm(has_one)]
        pub profile: HasOne<super::profile::Entity>,
        #[sea_orm(has_many)]
        pub posts: HasMany<super::post::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod profile {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "profile")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub picture: String,
        #[sea_orm(unique)]
        pub user_id: i32,
        #[sea_orm(belongs_to, from = "user_id", to = "id")]
        pub user: HasOne<super::user::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod post {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "post")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub user_id: i32,
        pub body: String,
        #[sea_orm(belongs_to, from = "user_id", to = "id")]
        pub author: HasOne<super::user::Entity>,
        #[sea_orm(has_many)]
        pub comments: HasMany<super::comment::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod comment {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "comment")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub comment: String,
        pub user_id: i32,
        pub post_id: i32,
        #[sea_orm(belongs_to, from = "user_id", to = "id")]
        pub user: HasOne<super::user::Entity>,
        #[sea_orm(belongs_to, from = "post_id", to = "id")]
        pub post: HasOne<super::post::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    let env = env_logger::Env::default().filter_or("RUST_LOG", "info,sea_orm=debug,sqlx=warn");
    env_logger::Builder::from_env(env).init();

    use sea_orm::entity::*;

    let db = &Database::connect("sqlite::memory:").await?;

    // it doesn't matter which order you register entities.
    // SeaORM figures out the foreign key dependencies and
    // creates the tables in the right order along with foreign keys
    db.get_schema_builder()
        .register(user::Entity)
        .register(profile::Entity)
        .register(post::Entity)
        .register(comment::Entity)
        .apply(db)
        .await?;

    let bob = user::ActiveModel {
        name: Set("Bob".into()),
        email: Set("bob@sea-ql.org".into()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    assert_eq!(
        bob,
        user::Entity::find_by_email("bob@sea-ql.org")
            .one(db)
            .await?
            .unwrap()
    );

    let profile = profile::ActiveModel {
        user_id: Set(bob.id),
        picture: Set("sports pose".into()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // query user with profile in a single query
    let user_with_profile = user::Entity::load().with(profile::Entity).one(db).await?;
    let user_with_profile = user_with_profile.unwrap();
    assert_eq!(user_with_profile.name, "Bob");
    assert_eq!(user_with_profile.profile.unwrap().picture, "sports pose");

    // try update user profile
    let mut profile = profile.into_active_model();
    profile.picture = Set("landscape".into());
    profile.save(db).await?;

    assert_eq!(
        profile::Entity::find_by_user_id(bob.id).all(db).await?[0].picture,
        "landscape"
    );

    // Bob writes some posts
    post::ActiveModel {
        user_id: Set(bob.id),
        body: Set("Lorem ipsum dolor sit amet, consectetur adipiscing elit".into()),
        ..Default::default()
    }
    .insert(db)
    .await?;
    post::ActiveModel {
        user_id: Set(bob.id),
        body: Set("Ut enim ad minim veniam, quis nostrud exercitation".into()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // find Bob's profile and his posts
    let bob_posts = user::Entity::load()
        .filter_by_id(bob.id)
        .with(profile::Entity)
        .with(post::Entity)
        .one(db)
        .await?
        .unwrap();

    assert_eq!(bob_posts.name, "Bob");
    assert_eq!(bob_posts.profile.unwrap().picture, "landscape");
    assert!(bob_posts.posts[0].body.starts_with("Lorem ipsum"));
    assert!(bob_posts.posts[1].body.starts_with("Ut enim ad"));

    // new user Alice comment on post
    let alice = user::ActiveModel {
        name: Set("Alice".into()),
        email: Set("alice@rust-lang.org".into()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    let alice_comment = comment::ActiveModel {
        comment: Set("nice post!".into()),
        post_id: Set(bob_posts.posts[0].id),
        user_id: Set(alice.id),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // find all posts along with comments
    let posts = post::Entity::load()
        .with(user::Entity)
        .with((comment::Entity, user::Entity))
        .all(db)
        .await?;

    assert!(posts[0].body.starts_with("Lorem ipsum"));
    assert_eq!(posts[0].author.as_ref().unwrap().name, "Bob");
    assert_eq!(posts[0].comments.len(), 1);
    assert_eq!(posts[0].comments[0].comment, "nice post!");
    assert_eq!(posts[0].comments[0].user.as_ref().unwrap().name, "Alice");

    assert!(posts[1].body.starts_with("Ut enim ad"));
    assert_eq!(posts[1].author.as_ref().unwrap().name, "Bob");
    assert_eq!(posts[1].comments.len(), 0);

    // delete the comment
    alice_comment.delete(db).await?;

    let post = post::Entity::load()
        .filter_by_id(posts[0].id)
        .with(comment::Entity)
        .one(db)
        .await?
        .unwrap();

    assert_eq!(post.comments.len(), 0);

    Ok(())
}
