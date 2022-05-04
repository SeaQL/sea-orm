use actix_web::{http, test};
use serde::Serialize;
use crate::sea_orm::{MockDatabase, MockExecResult, DatabaseBackend, Transaction};

#[derive(Serialize)]
struct PostForm {
    title: String,
    text: String,
}

#[cfg(test)]
#[actix_web::test]
async fn test_create() {
    use super::*;
    let post_db: post::Model = post::Model {
        id: 15.to_owned(),
        title: "title".to_owned(),
        text: "text".to_owned(),
    };

    let conn = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results(vec![vec![post_db.clone()]])
        .append_exec_results(vec![
            MockExecResult {
                last_insert_id: 15,
                rows_affected: 1,
            },
        ])
        .into_connection();
    let templates = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
    let state = web::Data::new(AppState { conn, templates });
    let app = test::init_service(App::new().app_data(state.clone()).service(create)).await;

    let form = web::Form(PostForm {
        title: "title".into(),
        text: "text".into(),
    });

    let req = test::TestRequest::post().set_form(&form).uri("/").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), http::StatusCode::FOUND);

    // the commenting the following will allow the test to pass
    // otherwise the borrow checker complains about using the conn variable after it is moved
    assert_eq!(
        conn.into_transaction_log(),
        vec![
            Transaction::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"INSERT INTO "posts" ("title", "text") VALUES ($1, $2) RETURNING "id", "title", "text""#,
                vec![1u64.into()]
            ),
        ],
    );
}
