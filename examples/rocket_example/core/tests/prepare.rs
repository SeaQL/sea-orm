use ::entity::post;
use sea_orm::*;

pub fn prepare_mock_db() -> DatabaseConnection {
    MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results(vec![
            // First query result
            vec![post::Model {
                id: 1,
                title: "Title A".to_owned(),
                text: "Text A".to_owned(),
            }],
            // Second query result
            vec![post::Model {
                id: 5,
                title: "Title C".to_owned(),
                text: "Text C".to_owned(),
            }],
        ])
        .into_connection()
}
