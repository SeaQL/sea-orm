use axum_example_service::{Mutation, Query};
use entity::post;
use sea_orm::{ConnectionTrait, Database, Schema};

#[tokio::test]
async fn main() {
    let db = &Database::connect("sqlite::memory:").await.unwrap();

    db.execute(&Schema::new(db.get_database_backend()).create_table_from_entity(post::Entity))
        .await
        .unwrap();

    {
        let post = Mutation::create_post(
            db,
            post::Model {
                id: 0,
                title: "Title A".to_owned(),
                text: "Text A".to_owned(),
            },
        )
        .await
        .unwrap();

        assert_eq!(
            post,
            post::ActiveModel {
                id: sea_orm::ActiveValue::Unchanged(1),
                title: sea_orm::ActiveValue::Unchanged("Title A".to_owned()),
                text: sea_orm::ActiveValue::Unchanged("Text A".to_owned())
            }
        );
    }

    {
        let post = Mutation::create_post(
            db,
            post::Model {
                id: 0,
                title: "Title B".to_owned(),
                text: "Text B".to_owned(),
            },
        )
        .await
        .unwrap();

        assert_eq!(
            post,
            post::ActiveModel {
                id: sea_orm::ActiveValue::Unchanged(2),
                title: sea_orm::ActiveValue::Unchanged("Title B".to_owned()),
                text: sea_orm::ActiveValue::Unchanged("Text B".to_owned())
            }
        );
    }

    {
        let post = Query::find_post_by_id(db, 1).await.unwrap().unwrap();

        assert_eq!(post.id, 1);
        assert_eq!(post.title, "Title A");
    }

    {
        let post = Mutation::update_post_by_id(
            db,
            1,
            post::Model {
                id: 1,
                title: "New Title A".to_owned(),
                text: "New Text A".to_owned(),
            },
        )
        .await
        .unwrap();

        assert_eq!(
            post,
            post::Model {
                id: 1,
                title: "New Title A".to_owned(),
                text: "New Text A".to_owned(),
            }
        );
    }

    {
        let result = Mutation::delete_post(db, 2).await.unwrap();

        assert_eq!(result.rows_affected, 1);
    }

    {
        let post = Query::find_post_by_id(db, 2).await.unwrap();
        assert!(post.is_none());
    }

    {
        let result = Mutation::delete_all_posts(db).await.unwrap();

        assert_eq!(result.rows_affected, 1);
    }
}
