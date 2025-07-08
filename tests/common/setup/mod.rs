use pretty_assertions::assert_eq;
use sea_orm::{
    ColumnTrait, ColumnType, ConnectOptions, ConnectionTrait, Database, DatabaseBackend,
    DatabaseConnection, DbBackend, DbConn, DbErr, EntityTrait, ExecResult, Iterable, Schema,
    Statement,
};
use sea_query::{
    SeaRc, Table, TableCreateStatement,
    extension::postgres::{Type, TypeCreateStatement},
};

pub async fn setup(base_url: &str, db_name: &str) -> DatabaseConnection {
    if cfg!(feature = "sqlx-mysql") {
        let url = format!("{base_url}/mysql");
        let db = Database::connect(&url).await.unwrap();
        let _drop_db_result = db
            .execute_raw(Statement::from_string(
                DatabaseBackend::MySql,
                format!("DROP DATABASE IF EXISTS `{db_name}`;"),
            ))
            .await;

        let _create_db_result = db
            .execute_raw(Statement::from_string(
                DatabaseBackend::MySql,
                format!("CREATE DATABASE `{db_name}`;"),
            ))
            .await;

        let url = format!("{base_url}/{db_name}");
        Database::connect(&url).await.unwrap()
    } else if cfg!(feature = "sqlx-postgres") {
        let url = format!("{base_url}/postgres");
        let db = Database::connect(&url).await.unwrap();
        let _drop_db_result = db
            .execute_raw(Statement::from_string(
                DatabaseBackend::Postgres,
                format!("DROP DATABASE IF EXISTS \"{db_name}\";"),
            ))
            .await;

        let _create_db_result = db
            .execute_raw(Statement::from_string(
                DatabaseBackend::Postgres,
                format!("CREATE DATABASE \"{db_name}\";"),
            ))
            .await;

        let url = format!("{base_url}/{db_name}");
        Database::connect(&url).await.unwrap()
    } else {
        let mut options: ConnectOptions = base_url.into();
        options.sqlx_logging(false);
        Database::connect(options).await.unwrap()
    }
}

pub async fn tear_down(base_url: &str, db_name: &str) {
    if cfg!(feature = "sqlx-mysql") {
        let url = format!("{base_url}/mysql");
        let db = Database::connect(&url).await.unwrap();
        let _ = db
            .execute_raw(Statement::from_string(
                DatabaseBackend::MySql,
                format!("DROP DATABASE IF EXISTS \"{db_name}\";"),
            ))
            .await;
    } else if cfg!(feature = "sqlx-postgres") {
        let url = format!("{base_url}/postgres");
        let db = Database::connect(&url).await.unwrap();
        let _ = db
            .execute_raw(Statement::from_string(
                DatabaseBackend::Postgres,
                format!("DROP DATABASE IF EXISTS \"{db_name}\";"),
            ))
            .await;
    };
}

pub async fn create_enum<E>(
    db: &DbConn,
    creates: &[TypeCreateStatement],
    entity: E,
) -> Result<(), DbErr>
where
    E: EntityTrait,
{
    let builder = db.get_database_backend();
    if builder == DbBackend::Postgres {
        for col in E::Column::iter() {
            let col_def = col.def();
            let col_type = col_def.get_column_type();
            if !matches!(col_type, ColumnType::Enum { .. }) {
                continue;
            }
            let name = match col_type {
                ColumnType::Enum { name, .. } => name,
                _ => unreachable!(),
            };
            db.execute(Type::drop().name(SeaRc::clone(name)).if_exists().cascade())
                .await?;
        }
    }

    let expect_stmts: Vec<Statement> = creates.iter().map(|stmt| builder.build(stmt)).collect();
    let schema = Schema::new(builder);
    let create_from_entity_stmts: Vec<Statement> = schema
        .create_enum_from_entity(entity)
        .iter()
        .map(|stmt| builder.build(stmt))
        .collect();

    assert_eq!(expect_stmts, create_from_entity_stmts);

    for stmt in creates.iter() {
        db.execute(stmt).await?;
    }

    Ok(())
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
    let schema = Schema::new(builder);
    assert_eq!(
        builder.build(&schema.create_table_from_entity(entity)),
        builder.build(create)
    );

    create_table_without_asserts(db, create).await
}

pub async fn create_table_with_index<E>(
    db: &DbConn,
    create: &TableCreateStatement,
    entity: E,
) -> Result<ExecResult, DbErr>
where
    E: EntityTrait,
{
    let res = create_table(db, create, entity).await?;
    let backend = db.get_database_backend();
    for stmt in Schema::new(backend).create_index_from_entity(entity) {
        db.execute(&stmt).await?;
    }
    Ok(res)
}

pub async fn create_table_from_entity<E>(db: &DbConn, entity: E) -> Result<ExecResult, DbErr>
where
    E: EntityTrait,
{
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);
    let stmt = schema.create_table_from_entity(entity);

    db.execute(&stmt).await
}

pub async fn create_table_without_asserts(
    db: &DbConn,
    create: &TableCreateStatement,
) -> Result<ExecResult, DbErr> {
    let builder = db.get_database_backend();
    if builder != DbBackend::Sqlite {
        let stmt = Table::drop()
            .table(create.get_table_name().unwrap().clone())
            .if_exists()
            .cascade()
            .take();
        db.execute(&stmt).await?;
    }
    db.execute(create).await
}

pub fn rust_dec<T: ToString>(v: T) -> rust_decimal::Decimal {
    use std::str::FromStr;
    rust_decimal::Decimal::from_str(&v.to_string()).unwrap()
}
