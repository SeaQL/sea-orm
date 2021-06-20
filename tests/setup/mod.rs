use sea_orm::{Database, DatabaseConnection};

pub async fn setup() -> DatabaseConnection {
    Database::connect("sqlite::memory:")
        .await
        .unwrap()
}