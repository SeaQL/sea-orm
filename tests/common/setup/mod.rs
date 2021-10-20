use pretty_assertions::assert_eq;
use sea_orm::{
    ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, DbBackend, DbConn, DbErr,
    EntityTrait, ExecResult, Schema, Statement,
};
use sea_query::{Alias, Table, TableCreateStatement};

pub async fn setup(base_url: &str, db_name: &str) -> DatabaseConnection {
    let db = if cfg!(feature = "sqlx-mysql") {
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
        let url = format!("{}/postgres", base_url);
        let db = Database::connect(&url).await.unwrap();
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
        Database::connect(&url).await.unwrap()
    } else {
        Database::connect(base_url).await.unwrap()
    };

    db
}

pub async fn tear_down(base_url: &str, db_name: &str) {
    if cfg!(feature = "sqlx-mysql") {
        let url = format!("{}/mysql", base_url);
        let db = Database::connect(&url).await.unwrap();
        let _ = db
            .execute(Statement::from_string(
                DatabaseBackend::MySql,
                format!("DROP DATABASE IF EXISTS \"{}\";", db_name),
            ))
            .await;
    } else if cfg!(feature = "sqlx-postgres") {
        let url = format!("{}/postgres", base_url);
        let db = Database::connect(&url).await.unwrap();
        let _ = db
            .execute(Statement::from_string(
                DatabaseBackend::Postgres,
                format!("DROP DATABASE IF EXISTS \"{}\";", db_name),
            ))
            .await;
    } else {
    };
}

pub async fn create_table<E>(
    db: &DbConn,
    create: &TableCreateStatement,
    entity: E,
) -> Result<ExecResult, DbErr>
where
    E: EntityTrait,
{
    let builder = db.get_database_backend();
    if builder != DbBackend::Sqlite {
        let stmt = builder.build(
            Table::drop()
                .table(Alias::new(create.get_table_name().unwrap().as_ref()))
                .if_exists()
                .cascade(),
        );
        db.execute(stmt).await?;
    }

    let stmt = builder.build(create);
    assert_eq!(
        builder.build(&Schema::create_table_from_entity(entity)),
        stmt
    );
    db.execute(stmt).await
}
