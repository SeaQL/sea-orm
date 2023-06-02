use ::entity::post;
use sea_orm::*;

#[cfg(feature = "mock")]
pub fn prepare_mock_db() -> DatabaseConnection {
    MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([
            [post::Model {
                id: 1,
                title: "Title A".to_owned(),
                text: "Text A".to_owned(),
            }],
            [post::Model {
                id: 5,
                title: "Title C".to_owned(),
                text: "Text C".to_owned(),
            }],
            [post::Model {
                id: 6,
                title: "Title D".to_owned(),
                text: "Text D".to_owned(),
            }],
            [post::Model {
                id: 1,
                title: "Title A".to_owned(),
                text: "Text A".to_owned(),
            }],
            [post::Model {
                id: 1,
                title: "New Title A".to_owned(),
                text: "New Text A".to_owned(),
            }],
            [post::Model {
                id: 5,
                title: "Title C".to_owned(),
                text: "Text C".to_owned(),
            }],
        ])
        .append_exec_results([
            MockExecResult {
                last_insert_id: 6,
                rows_affected: 1,
            },
            MockExecResult {
                last_insert_id: 6,
                rows_affected: 5,
            },
        ])
        .into_connection()
}
