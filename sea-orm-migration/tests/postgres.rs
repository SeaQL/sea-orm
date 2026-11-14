mod common;

#[cfg(all(test, feature = "sqlx-postgres"))]
mod inner {
    use crate::common::migrator::default::*;
    use sea_orm::{
        ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, IsolationLevel,
        Statement, TransactionTrait, error::DbErr,
    };
    use sea_orm_migration::{MigrationStatus, prelude::*};
    use std::time::Duration;

    #[tokio::test]
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

        // Create the extension, a custom type, and dependent objects.
        db.execute_unprepared("CREATE EXTENSION IF NOT EXISTS citext")
            .await?;
        db.execute_unprepared("CREATE TYPE \"UserFruit\" AS ENUM ('Apple', 'Banana')")
            .await?;
        db.execute_unprepared(r#"CREATE DOMAIN "UserFruitDomain" AS "UserFruit""#)
            .await?;
        db.execute_unprepared(
            r#"CREATE FUNCTION format_user_fruit("UserFruit") RETURNS text LANGUAGE SQL AS $$ SELECT $1::text $$"#,
        )
        .await?;

        // Run the fresh migration
        Migrator::fresh(&db).await?;

        // Check that the custom type and its dependent objects were dropped,
        // and the extension's type was not.
        let citext_exists: Option<i32> = db
            .query_one_raw(Statement::from_string(
                DbBackend::Postgres,
                r#"SELECT 1 as "value" FROM pg_type WHERE typname = 'citext'"#.to_owned(),
            ))
            .await?
            .map(|row| row.try_get("", "value").unwrap());

        assert_eq!(citext_exists, Some(1), "the citext type should still exist");

        let user_fruit_exists: Option<i32> = db
            .query_one_raw(Statement::from_string(
                DbBackend::Postgres,
                r#"SELECT 1 as "value" FROM pg_type WHERE typname = 'UserFruit'"#.to_owned(),
            ))
            .await?
            .map(|row| row.try_get("", "value").unwrap());

        assert_eq!(
            user_fruit_exists, None,
            "the UserFruit type should have been dropped"
        );

        let user_fruit_domain_exists: Option<i32> = db
            .query_one_raw(Statement::from_string(
                DbBackend::Postgres,
                r#"SELECT 1 as "value" FROM pg_type WHERE typname = 'UserFruitDomain'"#.to_owned(),
            ))
            .await?
            .map(|row| row.try_get("", "value").unwrap());

        assert_eq!(
            user_fruit_domain_exists, None,
            "the dependent UserFruitDomain type should have been dropped"
        );

        let format_user_fruit_exists: Option<i32> = db
            .query_one_raw(Statement::from_string(
                DbBackend::Postgres,
                r#"SELECT 1 as "value" FROM pg_proc WHERE proname = 'format_user_fruit'"#
                    .to_owned(),
            ))
            .await?
            .map(|row| row.try_get("", "value").unwrap());

        assert_eq!(
            format_user_fruit_exists, None,
            "the dependent format_user_fruit function should have been dropped"
        );

        Ok(())
    }

    /// A fresh database, or `None` when `DATABASE_URL` is not Postgres.
    async fn create_db(db_name: &str) -> Result<Option<String>, DbErr> {
        let url =
            &std::env::var("DATABASE_URL").expect("Environment variable 'DATABASE_URL' not set");
        let db = Database::connect(url).await?;
        if !matches!(db.get_database_backend(), DbBackend::Postgres) {
            return Ok(None);
        }
        db.execute_unprepared(&format!(r#"DROP DATABASE IF EXISTS "{db_name}""#))
            .await?;
        db.execute_unprepared(&format!(r#"CREATE DATABASE "{db_name}""#))
            .await?;
        Ok(Some(format!("{url}/{db_name}")))
    }

    /// Several instances of an app starting at once, each migrating on start.
    async fn concurrently<F, Fut>(url: &str, run: F) -> Result<(), DbErr>
    where
        F: Fn(DatabaseConnection) -> Fut,
        Fut: Future<Output = Result<(), DbErr>> + Send + 'static,
    {
        // Connect first, so that the runs start together.
        let mut dbs = Vec::new();
        for _ in 0..8 {
            dbs.push(Database::connect(url).await?);
        }
        let runs: Vec<_> = dbs.into_iter().map(|db| tokio::spawn(run(db))).collect();
        for run in runs {
            run.await.expect("migration task panicked")?;
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_up() -> Result<(), DbErr> {
        let Some(url) = create_db("test_concurrent_up").await? else {
            return Ok(());
        };

        concurrently(&url, |db| async move { Migrator::up(&db, None).await }).await?;
        let db = Database::connect(&url).await?;
        assert!(Migrator::get_pending_migrations(&db).await?.is_empty());

        concurrently(&url, |db| async move { Migrator::down(&db, None).await }).await?;
        assert!(Migrator::get_applied_migrations(&db).await?.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_up_in_transaction() -> Result<(), DbErr> {
        let Some(url) = create_db("test_concurrent_up_in_transaction").await? else {
            return Ok(());
        };

        // A failed statement would abort the caller's transaction, so `install` must not race.
        concurrently(&url, |db| async move {
            let txn = db.begin().await?;
            Migrator::up(&txn, None).await?;
            txn.commit().await
        })
        .await?;
        let db = Database::connect(&url).await?;
        assert!(Migrator::get_pending_migrations(&db).await?.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_up_and_down() -> Result<(), DbErr> {
        let Some(url) = create_db("test_concurrent_up_and_down").await? else {
            return Ok(());
        };
        let db = Database::connect(&url).await?;
        Migrator::up(&db, Some(3)).await?;

        let mut runs = Vec::new();
        for i in 0..8 {
            let db = Database::connect(&url).await?;
            runs.push(tokio::spawn(async move {
                if i % 2 == 0 {
                    Migrator::up(&db, None).await
                } else {
                    Migrator::down(&db, None).await
                }
            }));
        }
        for run in runs {
            // A run gives up once another one has moved the state past it.
            if let Err(err) = run.await.expect("migration task panicked") {
                assert!(err.to_string().contains("by another process"), "{err}");
            }
        }

        // Whatever won, the applied migrations are a prefix of the list.
        let statuses: Vec<_> = Migrator::get_migration_with_status(&db)
            .await?
            .iter()
            .map(|m| m.status())
            .collect();
        let applied = statuses
            .iter()
            .take_while(|s| **s == MigrationStatus::Applied)
            .count();
        assert!(
            statuses[applied..]
                .iter()
                .all(|s| *s == MigrationStatus::Pending),
            "{statuses:?}"
        );

        Ok(())
    }

    /// The default migrations, and one run outside of a transaction that records an unknown
    /// version while it is rolled back, after `down` has read the applied migrations.
    struct RecordsUnknownVersion;

    #[async_trait::async_trait]
    impl MigratorTrait for RecordsUnknownVersion {
        fn migrations() -> Vec<Box<dyn MigrationTrait>> {
            let mut migrations = Migrator::migrations();
            migrations.push(Box::new(InsertsUnknownVersion));
            migrations
        }
    }

    struct IgnoresUnknownVersion;

    #[async_trait::async_trait]
    impl MigratorTrait for IgnoresUnknownVersion {
        fn migrations() -> Vec<Box<dyn MigrationTrait>> {
            RecordsUnknownVersion::migrations()
        }

        fn ignore_missing_migrations() -> bool {
            true
        }
    }

    #[derive(DeriveMigrationName)]
    struct InsertsUnknownVersion;

    #[async_trait::async_trait]
    impl MigrationTrait for InsertsUnknownVersion {
        async fn up(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
            Ok(())
        }

        async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            manager
                .get_connection()
                .execute_unprepared(
                    "INSERT INTO seaql_migrations (version, applied_at) VALUES ('m99990101_000001_unknown', 0)",
                )
                .await?;
            Ok(())
        }

        fn use_transaction(&self) -> Option<bool> {
            Some(false)
        }
    }

    #[tokio::test]
    async fn test_down_rejects_unknown_migration() -> Result<(), DbErr> {
        let Some(url) = create_db("test_down_rejects_unknown_migration").await? else {
            return Ok(());
        };
        let db = Database::connect(&url).await?;
        RecordsUnknownVersion::up(&db, None).await?;

        let err = RecordsUnknownVersion::down(&db, Some(2)).await.unwrap_err();
        assert!(
            err.to_string()
                .contains("version 'm99990101_000001_unknown' is missing"),
            "{err}"
        );
        let applied = db
            .query_one_raw(Statement::from_string(
                DbBackend::Postgres,
                "SELECT 1 FROM seaql_migrations WHERE version = 'm20230109_000001_seed_cake_table'",
            ))
            .await?;
        assert!(
            applied.is_some(),
            "the last known migration was rolled back"
        );

        // A migrator that ignores missing migrations rolls back past the unknown version.
        IgnoresUnknownVersion::down(&db, Some(1)).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_rejects_snapshot_isolation() -> Result<(), DbErr> {
        let Some(url) = create_db("test_rejects_snapshot_isolation").await? else {
            return Ok(());
        };
        let db = Database::connect(&url).await?;

        for level in [IsolationLevel::RepeatableRead, IsolationLevel::Serializable] {
            let txn = db.begin_with_config(Some(level), None).await?;
            let err = Migrator::up(&txn, None).await.unwrap_err();
            assert!(err.to_string().contains("READ COMMITTED"), "{err}");
            let manager = SchemaManager::new(&txn);
            assert!(
                !manager.has_table("seaql_migrations").await?,
                "the migration table was created before the check"
            );
            assert!(
                !manager.has_table("cake").await?,
                "a migration ran before the check"
            );
            txn.rollback().await?;
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_lock_per_schema() -> Result<(), DbErr> {
        let Some(url) = create_db("test_lock_per_schema").await? else {
            return Ok(());
        };
        let connect = |schema: &str| {
            Database::connect(
                ConnectOptions::new(&url)
                    .set_schema_search_path(schema)
                    .to_owned(),
            )
        };
        let a = connect("a").await?;
        a.execute_unprepared("CREATE SCHEMA a; CREATE SCHEMA b")
            .await?;
        let b = connect("b").await?;

        // An unfinished transaction in schema `a` holds the lock until it commits.
        let txn = a.begin().await?;
        Migrator::install(&txn).await?;
        tokio::time::timeout(Duration::from_secs(5), Migrator::up(&b, None))
            .await
            .expect("schema `b` waited for the lock of schema `a`")?;
        txn.commit().await
    }
}
