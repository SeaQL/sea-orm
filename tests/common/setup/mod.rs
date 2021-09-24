use sea_orm::{Database, DatabaseConnection, DbBackend, Statement};
pub mod schema;
pub use schema::*;

pub async fn setup(base_url: &str, db_name: &str) -> DatabaseConnection {
    let db = if cfg!(feature = "sqlx-mysql") {
        let url = format!("{}/mysql", base_url);
        let db = Database::connect(&url).await.unwrap();
        let _drop_db_result = db
            .execute(Statement::from_string(
                DbBackend::MySql,
                format!("DROP DATABASE IF EXISTS `{}`;", db_name),
            ))
            .await;

        let _create_db_result = db
            .execute(Statement::from_string(
                DbBackend::MySql,
                format!("CREATE DATABASE `{}`;", db_name),
            ))
            .await;

        let url = format!("{}/{}", base_url, db_name);
        Database::connect(&url).await.unwrap()
    } else if cfg!(feature = "sqlx-postgres") {
        let url = format!("{}/postgres", base_url);
        let db = Database::connect(&url).await.unwrap();
        let _drop_db_result = db
            .execute(Statement::from_string(
                DbBackend::Postgres,
                format!("DROP DATABASE IF EXISTS \"{}\";", db_name),
            ))
            .await;

        let _create_db_result = db
            .execute(Statement::from_string(
                DbBackend::Postgres,
                format!("CREATE DATABASE \"{}\";", db_name),
            ))
            .await;

        let url = format!("{}/{}", base_url, db_name);
        Database::connect(&url).await.unwrap()
    } else {
        Database::connect(base_url).await.unwrap()
    };

    schema::create_bakery_table(&db).await.unwrap();
    schema::create_baker_table(&db).await.unwrap();
    schema::create_customer_table(&db).await.unwrap();
    schema::create_order_table(&db).await.unwrap();
    schema::create_cake_table(&db).await.unwrap();
    schema::create_cakes_bakers_table(&db).await.unwrap();
    schema::create_lineitem_table(&db).await.unwrap();
    schema::create_metadata_table(&db).await.unwrap();
    schema::create_log_table(&db).await.unwrap();
    db
}

pub async fn tear_down(base_url: &str, db_name: &str) {
    if cfg!(feature = "sqlx-mysql") {
        let url = format!("{}/mysql", base_url);
        let db = Database::connect(&url).await.unwrap();
        let _ = db
            .execute(Statement::from_string(
                DbBackend::MySql,
                format!("DROP DATABASE IF EXISTS \"{}\";", db_name),
            ))
            .await;
    } else if cfg!(feature = "sqlx-postgres") {
        let url = format!("{}/postgres", base_url);
        let db = Database::connect(&url).await.unwrap();
        let _ = db
            .execute(Statement::from_string(
                DbBackend::Postgres,
                format!("DROP DATABASE IF EXISTS \"{}\";", db_name),
            ))
            .await;
    } else {
    };
}
