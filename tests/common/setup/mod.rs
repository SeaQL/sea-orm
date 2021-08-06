use sea_orm::{Database, DatabaseBackend, DatabaseConnection, Statement};
pub mod schema;
pub use schema::*;

pub async fn setup(base_url: &str, db_name: &str) -> DatabaseConnection {
    let db = if cfg!(feature = "sqlx-mysql") {
        println!("sqlx-mysql");

        let url = format!("{}/mysql", base_url);
        let db = Database::connect(&url).await.unwrap();
        let _drop_db_result = db
            .execute(Statement::from_string(
                DatabaseBackend::MySql,
                format!("DROP DATABASE IF EXISTS `{}`;", db_name),
            ))
            .await;

        let _create_db_result = db
            .execute(Statement::from_string(
                DatabaseBackend::MySql,
                format!("CREATE DATABASE `{}`;", db_name),
            ))
            .await;

        let url = format!("{}/{}", base_url, db_name);
        Database::connect(&url).await.unwrap()
    } else if cfg!(feature = "sqlx-postgres") {
        println!("sqlx-postgres");

        let url = format!("{}/postgres", base_url);
        println!("url: {:#?}", url);
        let db = Database::connect(&url).await.unwrap();
        println!("db: {:#?}", db);
        let _drop_db_result = db
            .execute(Statement::from_string(
                DatabaseBackend::Postgres,
                format!("DROP DATABASE IF EXISTS \"{}\";", db_name),
            ))
            .await;

        let _create_db_result = db
            .execute(Statement::from_string(
                DatabaseBackend::Postgres,
                format!("CREATE DATABASE \"{}\";", db_name),
            ))
            .await;

        let url = format!("{}/{}", base_url, db_name);
        println!("url: {:#?}", url);

        Database::connect(&url).await.unwrap()
    } else {
        println!("sqlx-sqlite");

        Database::connect(base_url).await.unwrap()
    };

    assert!(schema::create_bakery_table(&db).await.is_ok());
    assert!(schema::create_baker_table(&db).await.is_ok());
    assert!(schema::create_customer_table(&db).await.is_ok());
    assert!(schema::create_order_table(&db).await.is_ok());
    assert!(schema::create_cake_table(&db).await.is_ok());
    assert!(schema::create_cakes_bakers_table(&db).await.is_ok());
    assert!(schema::create_lineitem_table(&db).await.is_ok());
    db
}

pub async fn tear_down(base_url: &str, db_name: &str) {
    if cfg!(feature = "sqlx-mysql") {
        println!("sqlx-mysql");

        let url = format!("{}/mysql", base_url);
        let db = Database::connect(&url).await.unwrap();
        let _ = db
            .execute(Statement::from_string(
                DatabaseBackend::MySql,
                format!("DROP DATABASE IF EXISTS \"{}\";", db_name),
            ))
            .await;
    } else if cfg!(feature = "sqlx-postgres") {
        println!("sqlx-postgres");

        let url = format!("{}/postgres", base_url);
        println!("url: {:#?}", url);
        let db = Database::connect(&url).await.unwrap();
        let _ = db
            .execute(Statement::from_string(
                DatabaseBackend::Postgres,
                format!("DROP DATABASE IF EXISTS \"{}\";", db_name),
            ))
            .await;
    } else {
        println!("sqlx-sqlite");
    };
}
