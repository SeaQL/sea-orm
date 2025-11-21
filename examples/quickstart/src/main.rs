use log::info;

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
        pub title: String,
        #[sea_orm(belongs_to, from = "user_id", to = "id")]
        pub author: HasOne<super::user::Entity>,
        #[sea_orm(has_many)]
        pub comments: HasMany<super::comment::Entity>,
        #[sea_orm(has_many, via = "post_tag")]
        pub tags: HasMany<super::tag::Entity>,
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

mod tag {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "tag")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(unique)]
        pub tag: String,
        #[sea_orm(has_many, via = "post_tag")]
        pub posts: HasMany<super::post::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod post_tag {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "post_tag")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub post_id: i32,
        #[sea_orm(primary_key, auto_increment = false)]
        pub tag_id: i32,
        #[sea_orm(belongs_to, from = "post_id", to = "id")]
        pub post: Option<super::post::Entity>,
        #[sea_orm(belongs_to, from = "tag_id", to = "id")]
        pub tag: Option<super::tag::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

#[tokio::main]
async fn main() -> Result<(), sea_orm::DbErr> {
    ///// Part 0: Setup Environment /////

    // This disables sqlx's logging and enables sea-orm's logging with parameter injection,
    // which is easier to debug.
    let env = env_logger::Env::default().filter_or("RUST_LOG", "info,sea_orm=debug,sqlx=warn");
    env_logger::Builder::from_env(env).init();

    use sea_orm::{entity::*, query::*};

    // Use a SQLite in memory database so no setup needed.
    // SeaORM supports MySQL, Postgres, SQL Server as well.
    let db = &sea_orm::Database::connect("sqlite::memory:").await?;

    // Populate this fresh database with tables.
    //
    // All entities defined in this crate are automatically registered
    // into the schema registry, regardless of which module they live in.
    //
    // The registry may also include entities from upstream crates,
    // so here we restrict it to entities defined in this crate only.
    //
    // The order of entity definitions does not matter.
    // SeaORM resolves foreign key dependencies automatically
    // and creates the tables in the correct order with their keys.
    db.get_schema_registry("sea_orm_quickstart::*")
        .sync(db)
        .await?;

    info!("Schema created.");

    ///// Part 1: CRUD with nested 1-1 and 1-N relations /////

    info!("Create user Bob with a profile:");
    let bob = user::ActiveModel::builder()
        .set_name("Bob")
        .set_email("bob@sea-ql.org")
        .set_profile(profile::ActiveModel::builder().set_picture("Tennis"))
        .insert(db)
        .await?;

    info!("Find Bob by email:");
    assert_eq!(
        bob,
        // this method is generated by #[sea_orm::model] on unique keys
        user::Entity::find_by_email("bob@sea-ql.org")
            .one(db)
            .await?
            .unwrap()
    );

    info!("Query user with profile in a single query:");
    let mut bob = user::Entity::load()
        .filter_by_id(bob.id)
        .with(profile::Entity)
        .one(db)
        .await?
        .expect("Not found");
    assert_eq!(bob.name, "Bob");
    assert_eq!(bob.profile.as_ref().unwrap().picture, "Tennis");

    // Here we take ownership of the nested model, modify in place and save it
    info!("Update Bob's profile:");
    bob.profile
        .take()
        .unwrap()
        .into_active_model()
        .set_picture("Landscape")
        .save(db)
        .await?;

    info!("Confirmed that it's been updated:");
    assert_eq!(
        profile::Entity::find_by_user_id(bob.id).all(db).await?[0].picture,
        "Landscape"
    );

    // we don't have to set the `user_id` of the posts, they're automatically set to Bob
    info!("Bob wrote some posts:");
    let mut bob = bob.into_active_model();
    bob.posts
        .push(
            post::ActiveModel::builder()
                .set_title("Lorem ipsum dolor sit amet, consectetur adipiscing elit"),
        )
        .push(
            post::ActiveModel::builder()
                .set_title("Ut enim ad minim veniam, quis nostrud exercitation"),
        );
    bob.save(db).await?;

    info!("Find Bob's profile and his posts:");
    let bob = user::Entity::load()
        .filter(user::COLUMN.name.eq("Bob"))
        .with(profile::Entity)
        .with(post::Entity)
        .one(db)
        .await?
        .unwrap();

    assert_eq!(bob.name, "Bob");
    assert_eq!(bob.profile.as_ref().unwrap().picture, "Landscape");
    assert!(bob.posts[0].title.starts_with("Lorem ipsum"));
    assert!(bob.posts[1].title.starts_with("Ut enim ad"));

    // It's actually fine to create user + profile the other way round.
    // SeaORM figures out the dependency and creates the user first.
    info!("Create a new user Alice:");
    let alice = profile::ActiveModel::builder()
        .set_user(
            user::ActiveModel::builder()
                .set_name("Alice")
                .set_email("alice@rust-lang.org"),
        )
        .set_picture("Park")
        .insert(db)
        .await?
        .user
        .unwrap();

    // Not only can we insert new posts via the bob active model,
    // we can also add new comments to the posts.
    // SeaORM walks the document tree and figures out what's changed,
    // and perform the operation in one transaction.
    let mut bob = bob.into_active_model();
    info!("Alice commented on Bob's post:");
    bob.posts[0].comments.push(
        comment::ActiveModel::builder()
            .set_comment("nice post!")
            .set_user_id(alice.id),
    );
    bob.posts[1].comments.push(
        comment::ActiveModel::builder()
            .set_comment("interesting!")
            .set_user_id(alice.id),
    );
    bob.save(db).await?;

    info!("Find all posts with author along with comments and who commented:");
    let posts = post::Entity::load()
        .with(user::Entity)
        .with((comment::Entity, user::Entity))
        .all(db)
        .await?;

    assert!(posts[0].title.starts_with("Lorem ipsum"));
    assert_eq!(posts[0].author.as_ref().unwrap().name, "Bob");
    assert_eq!(posts[0].comments.len(), 1);
    assert_eq!(posts[0].comments[0].comment, "nice post!");
    assert_eq!(posts[0].comments[0].user.as_ref().unwrap().name, "Alice");

    assert!(posts[1].title.starts_with("Ut enim ad"));
    assert_eq!(posts[1].author.as_ref().unwrap().name, "Bob");
    assert_eq!(posts[1].comments.len(), 1);
    assert_eq!(posts[1].comments[0].comment, "interesting!");

    // Again, we can apply multiple changes in one operation,
    // the queries are executed inside a transaction.
    info!("Update post title and comment on first post:");
    let mut post = posts[0].clone().into_active_model();
    post.title = Set("Lorem ipsum dolor sit amet".into()); // shorten it
    post.comments[0].comment = Set("nice post! I learnt a lot".into());
    post.save(db).await?;

    info!("Confirm the post and comment is updated");
    let post = post::Entity::load()
        .filter_by_id(posts[0].id)
        .with(comment::Entity)
        .one(db)
        .await?
        .unwrap();

    assert_eq!(post.title, "Lorem ipsum dolor sit amet");
    assert_eq!(post.comments[0].comment, "nice post! I learnt a lot");

    // Comments belongs to post. They will be deleted first, otherwise the foreign key
    // would prevent the operation.
    info!("Delete the post along with all comments");
    post.delete(db).await?;

    assert!(
        post::Entity::find_by_id(posts[0].id)
            .one(db)
            .await?
            .is_none()
    );

    ///// Part 2: managing M-N relations /////

    // A unique feature of SeaORM is modelling many-to-many relations in a high level way

    info!("Insert one tag for later use");
    let sunny = tag::ActiveModel::builder()
        .set_tag("sunny")
        .save(db)
        .await?;

    info!("Insert a new post with 2 tags");
    let mut post = post::ActiveModel::builder()
        .set_title("A perfect day out")
        .set_user_id(alice.id)
        .add_tag(sunny.clone()) // an existing tag
        .add_tag(tag::ActiveModel::builder().set_tag("foodie")) // a new tag
        .save(db) // new tag will be created and associcated to the new post
        .await?;

    let post_id = post.id.clone().unwrap();

    {
        info!("get back the post and tags");
        let post = post::Entity::load()
            .filter_by_id(post_id)
            .with(tag::Entity)
            .one(db)
            .await?
            .unwrap();
        assert_eq!(post.title, "A perfect day out");
        assert_eq!(post.tags.len(), 2);
        assert_eq!(post.tags[0].tag, "sunny");
        assert_eq!(post.tags[1].tag, "foodie");
    }

    info!("Add new tag to post");
    post.tags
        .push(tag::ActiveModel::builder().set_tag("downtown"));
    let mut post = post.save(db).await?;

    {
        info!("get back the post and tags");
        let post = post::Entity::load()
            .filter_by_id(post_id)
            .with(tag::Entity)
            .one(db)
            .await?
            .unwrap();
        assert_eq!(post.tags.len(), 3);
        assert_eq!(post.tags[0].tag, "sunny");
        assert_eq!(post.tags[1].tag, "foodie");
        assert_eq!(post.tags[2].tag, "downtown");
    }

    info!("Update post title and remove a tag");
    let mut tags = post.tags.take();
    tags.as_mut_vec().remove(0); // it actually rained
    post.title = Set("Almost a perfect day out".into());
    post.tags.replace_all(tags);
    // converting the field from append to replace would delete associations not in this list
    let post = post.save(db).await?;

    {
        info!("get back the post and tags");
        let post = post::Entity::load()
            .filter_by_id(post_id)
            .with(tag::Entity)
            .one(db)
            .await?
            .unwrap();
        assert_eq!(post.tags.len(), 2);
        assert_eq!(post.title, "Almost a perfect day out");
        assert_eq!(post.tags[0].tag, "foodie");
        assert_eq!(post.tags[1].tag, "downtown");
    }

    // only the association between post and tag is removed,
    // but the tag itself is not deleted
    info!("check that the tag sunny still exists");
    assert!(tag::Entity::find_by_tag("sunny").one(db).await?.is_some());

    info!("cascade delete post, remove tag associations");
    post.delete(db).await?;

    Ok(())
}
