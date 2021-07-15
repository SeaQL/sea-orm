mod entity;

use entity::*;

use sea_orm::{entity::*, error::*, DatabaseBackend, MockDatabase, MockExecResult, Transaction};

#[async_std::test]
async fn test_insert() -> Result<(), DbErr> {
    let exec_result = MockExecResult {
        last_insert_id: 1,
        rows_affected: 1,
    };

    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results(vec![exec_result.clone()])
        .into_connection();

    let apple = cake::ActiveModel {
        name: Set("Apple Pie".to_owned()),
        ..Default::default()
    };

    let insert_result = cake::Entity::insert(apple).exec(&db).await?;

    assert_eq!(insert_result.last_insert_id, exec_result.last_insert_id);

    assert_eq!(
        db.into_transaction_log(),
        vec![Transaction::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"INSERT INTO "cake" ("name") VALUES ($1)"#,
            vec!["Apple Pie".into()]
        )]
    );

    Ok(())
}

#[async_std::test]
async fn test_select() -> Result<(), DbErr> {
    let query_results = vec![cake_filling::Model {
        cake_id: 2,
        filling_id: 3,
    }];

    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results(vec![query_results.clone()])
        .into_connection();

    let selected_models = cake_filling::Entity::find_by_id((2, 3)).all(&db).await?;

    assert_eq!(selected_models, query_results);

    assert_eq!(
        db.into_transaction_log(),
        vec![Transaction::from_sql_and_values(
            DatabaseBackend::Postgres,
            [
                r#"SELECT "cake_filling"."cake_id", "cake_filling"."filling_id" FROM "cake_filling""#,
                r#"WHERE "cake_filling"."cake_id" = $1 AND "cake_filling"."filling_id" = $2"#,
            ].join(" ").as_str(),
            vec![2i32.into(), 3i32.into()]
        )]
    );

    Ok(())
}

#[async_std::test]
async fn test_update() -> Result<(), DbErr> {
    let exec_result = MockExecResult {
        last_insert_id: 1,
        rows_affected: 1,
    };

    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results(vec![exec_result.clone()])
        .into_connection();

    let orange = fruit::ActiveModel {
        id: Set(1),
        name: Set("Orange".to_owned()),
        ..Default::default()
    };

    let updated_model = fruit::Entity::update(orange.clone()).exec(&db).await?;

    assert_eq!(updated_model, orange);

    assert_eq!(
        db.into_transaction_log(),
        vec![Transaction::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"UPDATE "fruit" SET "name" = $1 WHERE "fruit"."id" = $2"#,
            vec!["Orange".into(), 1i32.into()]
        )]
    );

    Ok(())
}

#[async_std::test]
async fn test_delete() -> Result<(), DbErr> {
    let exec_result = MockExecResult {
        last_insert_id: 1,
        rows_affected: 1,
    };

    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results(vec![exec_result.clone()])
        .into_connection();

    let orange = fruit::ActiveModel {
        id: Set(3),
        ..Default::default()
    };

    let delete_result = fruit::Entity::delete(orange).exec(&db).await?;

    assert_eq!(delete_result.rows_affected, exec_result.rows_affected);

    assert_eq!(
        db.into_transaction_log(),
        vec![Transaction::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"DELETE FROM "fruit" WHERE "fruit"."id" = $1"#,
            vec![3i32.into()]
        )]
    );

    Ok(())
}
