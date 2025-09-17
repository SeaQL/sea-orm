mod common;

#[cfg(all(test, feature = "sqlx-postgres"))]
mod inner {
    use crate::common::migrator::default::*;
    use sea_orm::{error::DbErr, ConnectOptions, ConnectionTrait, Database, DbBackend, Statement};
    use sea_orm_migration::prelude::*;

    #[async_std::test]
    async fn test_fresh_with_extension() -> Result<(), DbErr> {
        let url =
            &std::env::var("DATABASE_URL").expect("Environment variable 'DATABASE_URL' not set");
        let db_name = "test_fresh_with_extension";

        let db_connect = |url: String| async {
            let connect_options = ConnectOptions::new(url).to_owned();
            Database::connect(connect_options).await
        };

        let db = db_connect(url.to_owned()).await?;
        if !matches!(db.get_database_backend(), DbBackend::Postgres) {
            return Ok(());
        }

        db.execute_unprepared(&format!(r#"DROP DATABASE IF EXISTS "{db_name}""#))
            .await?;
        db.execute_unprepared(&format!(r#"CREATE DATABASE "{db_name}""#))
            .await?;

        let url = format!("{url}/{db_name}");
        let db = db_connect(url).await?;

        // Create the extension and a custom type
        db.execute_unprepared("CREATE EXTENSION IF NOT EXISTS citext")
            .await?;
        db.execute_unprepared("CREATE TYPE \"UserFruit\" AS ENUM ('Apple', 'Banana')")
            .await?;

        // Run the fresh migration
        Migrator::fresh(&db).await?;

        // Check that the custom type was dropped and the extension's type was not
        let citext_exists: Option<i32> = db
            .query_one(Statement::from_string(
                DbBackend::Postgres,
                r#"SELECT 1 as "value" FROM pg_type WHERE typname = 'citext'"#.to_owned(),
            ))
            .await?
            .map(|row| row.try_get("", "value").unwrap());

        assert_eq!(citext_exists, Some(1), "the citext type should still exist");

        let user_fruit_exists: Option<i32> = db
            .query_one(Statement::from_string(
                DbBackend::Postgres,
                r#"SELECT 1 as "value" FROM pg_type WHERE typname = 'UserFruit'"#.to_owned(),
            ))
            .await?
            .map(|row| row.try_get("", "value").unwrap());

        assert_eq!(
            user_fruit_exists, None,
            "the UserFruit type should have been dropped"
        );

        Ok(())
    }
}
