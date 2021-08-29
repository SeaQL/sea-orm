use rocket::figment::Figment;
use rocket_db_pools::{Config, Error};

#[rocket::async_trait]
impl rocket_db_pools::Pool for crate::Database {
    type Error = crate::DbErr;

    type Connection = crate::DatabaseConnection;

    async fn init(figment: &Figment) -> Result<Self, Self::Error> {
        Ok(crate::Database {})
    }

    async fn get(&self) -> Result<Self::Connection, Self::Error> {
        #[cfg(feature = "sqlx-mysql")]
        let db_url = "mysql://root:@localhost/rocket_example";
        #[cfg(feature = "sqlx-postgres")]
        let db_url = "postgres://root:root@localhost/rocket_example";

        println!("db_url: {:#?}", db_url);

        Ok(crate::Database::connect(db_url).await.unwrap())
    }
}
