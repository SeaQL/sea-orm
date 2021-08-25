use rocket::figment::Figment;
use rocket_db_pools::{Config, Error};

#[rocket::async_trait]
impl rocket_db_pools::Pool for crate::Database {
    type Error = crate::DbErr;

    type Connection = crate::DatabaseConnection;

    async fn init(figment: &Figment) -> Result<Self, Self::Error> {
        // let config = figment.extract::<Config>()?;
        // let mut opts = config.url.parse::<Options<D>>().map_err(Error::Init)?;
        // opts.disable_statement_logging();
        // specialize(&mut opts, &config);

        // sqlx::pool::PoolOptions::new()
        //     .max_connections(config.max_connections as u32)
        //     .connect_timeout(Duration::from_secs(config.connect_timeout))
        //     .idle_timeout(config.idle_timeout.map(Duration::from_secs))
        //     .min_connections(config.min_connections.unwrap_or_default())
        //     .connect_with(opts)
        //     .await
        //     .map_err(Error::Init)
        Ok(crate::Database {})
    }

    async fn get(&self) -> Result<Self::Connection, Self::Error> {
        // self.acquire().await.map_err(Error::Get)
        // let con = crate::Database::connect("sqlite::memory:").await;

        // Ok(crate::Database::connect("sqlite::memory:").await.unwrap())
        // "mysql://root:@localhost"
        Ok(
            crate::Database::connect("mysql://root:@localhost/rocket_example")
                .await
                .unwrap(),
        )
    }
}
