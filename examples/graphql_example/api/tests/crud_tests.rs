use entity::note;
use graphql_example_api::service::{Mutation, Query};
use sea_orm::{ConnectionTrait, Database, Schema};

#[tokio::test]
async fn crud_tests() {
    let db = &Database::connect("sqlite::memory:").await.unwrap();

    db.execute(&Schema::new(db.get_database_backend()).create_table_from_entity(note::Entity))
        .await
        .unwrap();

    {
        let note = Mutation::create_note(
            db,
            note::Model {
                id: 0,
                title: "Title A".to_owned(),
                text: "Text A".to_owned(),
            },
        )
        .await
        .unwrap();

        assert_eq!(
            note,
            note::Model {
                id: 1,
                title: "Title A".to_owned(),
                text: "Text A".to_owned(),
            }
        );
    }

    {
        let note = Mutation::create_note(
            db,
            note::Model {
                id: 0,
                title: "Title B".to_owned(),
                text: "Text B".to_owned(),
            },
        )
        .await
        .unwrap();

        assert_eq!(
            note,
            note::Model {
                id: 2,
                title: "Title B".to_owned(),
                text: "Text B".to_owned(),
            }
        );
    }

    {
        let note = Query::find_note_by_id(db, 1).await.unwrap().unwrap();

        assert_eq!(note.id, 1);
        assert_eq!(note.title, "Title A");
    }

    {
        let note = Mutation::update_note_by_id(
            db,
            1,
            note::Model {
                id: 1,
                title: "New Title A".to_owned(),
                text: "New Text A".to_owned(),
            },
        )
        .await
        .unwrap();

        assert_eq!(
            note,
            note::Model {
                id: 1,
                title: "New Title A".to_owned(),
                text: "New Text A".to_owned(),
            }
        );
    }

    {
        let result = Mutation::delete_note(db, 2).await.unwrap();

        assert_eq!(result.rows_affected, 1);
    }

    {
        let note = Query::find_note_by_id(db, 2).await.unwrap();
        assert!(note.is_none());
    }

    {
        let result = Mutation::delete_all_notes(db).await.unwrap();

        assert_eq!(result.rows_affected, 1);
    }
}
