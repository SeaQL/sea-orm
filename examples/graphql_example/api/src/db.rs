use graphql_example_core::sea_orm::DatabaseConnection;

pub struct Database {
    pub connection: DatabaseConnection,
}

impl Database {
    pub async fn new() -> Self {
        let connection = graphql_example_core::sea_orm::Database::connect(
            std::env::var("DATABASE_URL").unwrap(),
        )
        .await
        .expect("Could not connect to database");

        Database { connection }
    }

    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.connection
    }
}
