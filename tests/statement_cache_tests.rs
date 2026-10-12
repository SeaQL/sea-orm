#[cfg(all(feature = "mock", feature = "macros"))]
mod mock {
    use sea_orm::{tests_cfg::cake, *};

    fn query() -> Select<cake::Entity> {
        cake::Entity::find().filter(cake::Column::Id.eq(1))
    }

    fn model() -> cake::Model {
        cake::Model {
            id: 1,
            name: "cake".into(),
        }
    }

    #[test]
    fn raw_sql_defaults_to_persistent_and_keeps_parameters_bound() {
        let name = "a'; SELECT 1; --";
        let stmt = raw_sql!(Postgres, "SELECT {name}");
        assert!(stmt.persistent);
        assert_eq!(stmt.sql, "SELECT $1");
        assert_eq!(stmt.values, Some(Values(vec![name.into()])));
        assert!(!stmt.persistent(false).persistent);
    }

    #[cfg_attr(feature = "sync", test)]
    #[cfg_attr(not(feature = "sync"), smol_potat::test)]
    async fn selectors_preserve_bindings_and_cache_policy() -> Result<(), DbErr> {
        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[model()], [model()], [model()]])
            .into_connection();
        assert_eq!(query().persistent(false).all(&db).await?, [model()]);
        assert_eq!(query().persistent(false).one(&db).await?, Some(model()));
        assert_eq!(query().all(&db).await?, [model()]);
        let one = query().limit(1);
        assert_eq!(
            db.into_transaction_log(),
            Transaction::wrap([
                query().build(DbBackend::Postgres).persistent(false),
                one.build(DbBackend::Postgres).persistent(false),
                query().build(DbBackend::Postgres),
            ])
        );
        Ok(())
    }

    #[cfg_attr(feature = "sync", test)]
    #[cfg_attr(not(feature = "sync"), smol_potat::test)]
    async fn pagination_and_raw_selectors_preserve_cache_policy() -> Result<(), DbErr> {
        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[model()], [model()]])
            .append_query_results([[maplit::btreemap! { "num_items" => 1i64.into() }]])
            .append_query_results([[maplit::btreemap! { "num_items" => 1i64.into() }]])
            .into_connection();
        assert_eq!(
            query()
                .persistent(false)
                .paginate(&db, 2)
                .fetch_page(0)
                .await?,
            [model()]
        );
        let stmt = query().build(DbBackend::Postgres);
        assert_eq!(
            cake::Entity::find()
                .from_raw_sql(stmt)
                .persistent(false)
                .paginate(&db, 2)
                .fetch_page(0)
                .await?,
            [model()]
        );
        assert_eq!(
            query()
                .persistent(false)
                .paginate(&db, 2)
                .num_items()
                .await?,
            1
        );
        assert_eq!(query().persistent(false).count(&db).await?, 1);
        let log = db.into_transaction_log();
        assert_eq!(log.len(), 4);
        assert!(
            log.iter()
                .flat_map(Transaction::statements)
                .all(|s| !s.persistent)
        );
        assert!(
            log.iter().flat_map(Transaction::statements).all(|s| s
                .values
                .as_ref()
                .unwrap()
                .0
                .contains(&1i32.into()))
        );
        Ok(())
    }

    #[cfg_attr(feature = "sync", test)]
    #[cfg_attr(not(feature = "sync"), smol_potat::test)]
    #[cfg(all(feature = "stream", not(feature = "sync")))]
    async fn streaming_preserves_cache_policy() -> Result<(), DbErr> {
        use futures_util::TryStreamExt;
        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[model()]])
            .into_connection();
        let rows: Vec<_> = query()
            .persistent(false)
            .stream(&db)
            .await?
            .try_collect()
            .await?;
        assert_eq!(rows, [model()]);
        assert_eq!(
            db.into_transaction_log(),
            Transaction::wrap([query().build(DbBackend::Postgres).persistent(false),])
        );
        Ok(())
    }

    #[test]
    #[cfg(feature = "rbac")]
    fn cache_policy_preserves_query_auditing() {
        let query = query().into_query();
        let expected = StatementBuilder::audit(&query).unwrap();
        let actual = query.persistent(false).audit().unwrap();
        assert_eq!(actual.requests.len(), expected.requests.len());
        for (actual, expected) in actual.requests.iter().zip(&expected.requests) {
            assert_eq!(actual.access_type, expected.access_type);
            assert_eq!(actual.schema_table, expected.schema_table);
        }
    }
}

#[cfg(not(feature = "sync"))]
#[cfg(all(feature = "sqlx-postgres", feature = "runtime-tokio"))]
mod postgres {
    use sea_orm::{sqlx::Connection, *};

    mod setting {
        use sea_orm::entity::prelude::*;
        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "pg_settings", schema_name = "pg_catalog")]
        pub struct Model {
            #[sea_orm(primary_key, auto_increment = false)]
            pub name: String,
            pub setting: String,
        }
        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}
        impl ActiveModelBehavior for ActiveModel {}
    }

    #[tokio::test]
    async fn nonpersistent_queries_across_pool_connections() -> Result<(), DbErr> {
        async fn run_queries(db: &DatabaseConnection, offset: i32) -> Result<(), DbErr> {
            for _ in 0..16 {
                let stmt = Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    format!("SELECT $1::integer + {offset} AS value"),
                    [offset.into()],
                )
                .persistent(false);
                let row = db.query_one_raw(stmt).await?.unwrap();
                assert_eq!(row.try_get::<i32>("", "value")?, 2 * offset);
            }
            Ok(())
        }

        let url =
            std::env::var("DATABASE_URL").expect("set DATABASE_URL to a test PostgreSQL database");
        let mut options = ConnectOptions::new(url);
        options
            .max_connections(4)
            .min_connections(4)
            .set_schema_search_path("public")
            .sqlx_logging(false);
        let db = Database::connect(options).await?;
        let results =
            futures_util::future::join_all((0i32..16).map(|offset| run_queries(&db, offset))).await;
        db.close().await?;
        for result in results {
            result?;
        }
        Ok(())
    }

    async fn cache_size(db: &DatabaseConnection) -> usize {
        db.get_postgres_connection_pool()
            .acquire()
            .await
            .unwrap()
            .cached_statements_size()
    }

    /// Uses only catalog reads; no database or table is created or modified.
    #[tokio::test]
    async fn nonpersistent_queries_do_not_populate_the_postgres_statement_cache()
    -> Result<(), DbErr> {
        let url =
            std::env::var("DATABASE_URL").expect("set DATABASE_URL to a test PostgreSQL database");
        let mut options = ConnectOptions::new(url);
        options
            .max_connections(1)
            .min_connections(1)
            .sqlx_logging(false);
        let db = Database::connect(options).await?;
        let initial = cache_size(&db).await;
        let query = setting::Entity::find().filter(setting::Column::Name.eq("plan_cache_mode"));
        let expected = query.clone().persistent(false).require_one(&db).await?;
        for _ in 0..8 {
            assert_eq!(
                query.clone().persistent(false).all(&db).await?,
                [expected.clone()]
            );
        }
        assert_eq!(cache_size(&db).await, initial);
        let txn = db.begin().await?;
        assert_eq!(
            query.clone().persistent(false).one(&txn).await?,
            Some(expected.clone())
        );
        txn.commit().await?;
        assert_eq!(cache_size(&db).await, initial);
        assert_eq!(
            query
                .clone()
                .persistent(false)
                .paginate(&db, 2)
                .fetch_page(0)
                .await?,
            [expected.clone()]
        );
        assert_eq!(query.clone().persistent(false).count(&db).await?, 1);
        #[cfg(feature = "stream")]
        {
            use futures_util::TryStreamExt;
            let rows: Vec<_> = query
                .clone()
                .persistent(false)
                .stream(&db)
                .await?
                .try_collect()
                .await?;
            assert_eq!(rows, [expected.clone()]);
        }
        assert_eq!(cache_size(&db).await, initial);
        query.clone().all(&db).await?;
        assert_eq!(cache_size(&db).await, initial + 1);
        // Match SQLx semantics: opting out does not evict a previously cached statement.
        assert_eq!(query.persistent(false).all(&db).await?, [expected]);
        assert_eq!(cache_size(&db).await, initial + 1);
        db.close().await?;
        Ok(())
    }
}
#[cfg(not(feature = "sync"))]
#[cfg(all(feature = "sqlx-sqlite", feature = "runtime-tokio"))]
mod sqlite {
    use sea_orm::{sqlx::Connection, tests_cfg::cake, *};

    #[tokio::test]
    async fn nonpersistent_raw_queries_do_not_populate_the_sqlite_statement_cache()
    -> Result<(), DbErr> {
        let mut options = ConnectOptions::new("sqlite::memory:");
        options
            .max_connections(1)
            .min_connections(1)
            .sqlx_logging(false);
        let db = Database::connect(options).await?;
        let id = 7;
        let name = "cake";
        let stmt = raw_sql!(Sqlite, "SELECT {id} AS id, {name} AS name");
        let expected = cake::Model {
            id,
            name: name.into(),
        };
        let pool = db.get_sqlite_connection_pool();
        let initial = pool.acquire().await.unwrap().cached_statements_size();
        for _ in 0..8 {
            assert_eq!(
                cake::Entity::find()
                    .from_raw_sql(stmt.clone())
                    .persistent(false)
                    .all(&db)
                    .await?,
                [expected.clone()]
            );
        }
        assert_eq!(
            pool.acquire().await.unwrap().cached_statements_size(),
            initial
        );
        assert_eq!(
            cake::Entity::find().from_raw_sql(stmt).one(&db).await?,
            Some(expected)
        );
        assert_eq!(
            pool.acquire().await.unwrap().cached_statements_size(),
            initial + 1
        );
        db.close().await?;
        Ok(())
    }
}
