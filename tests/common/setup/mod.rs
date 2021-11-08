use pretty_assertions::assert_eq;
use sea_orm::{
    ColumnTrait, ColumnType, ConnectionTrait, Database, DatabaseBackend, DatabaseConnection,
    DbBackend, DbConn, DbErr, EntityTrait, ExecResult, Iterable, Schema, Statement,
};
use sea_query::{
    extension::postgres::{Type, TypeCreateStatement},
    Alias, Table, TableCreateStatement,
};

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
            if !matches!(col_type, ColumnType::Enum(_, _)) {
                continue;
            }
            let name = match col_type {
                ColumnType::Enum(s, _) => s.as_str(),
                _ => unreachable!(),
            };
            let drop_type_stmt = Type::drop()
                .name(Alias::new(name))
                .if_exists()
                .cascade()
                .to_owned();
            let stmt = builder.build(&drop_type_stmt);
            db.execute(stmt).await?;
        }
    }

    let expect_stmts: Vec<Statement> = creates.iter().map(|stmt| builder.build(stmt)).collect();
    let create_from_entity_stmts: Vec<Statement> = Schema::create_enum_from_entity(entity, builder)
        .iter()
        .map(|stmt| builder.build(stmt))
        .collect();

    assert_eq!(expect_stmts, create_from_entity_stmts);

    for stmt in expect_stmts {
        db.execute(stmt).await.map(|_| ())?;
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
    assert_eq!(
        builder.build(&Schema::create_table_from_entity(entity, builder)),
        builder.build(create)
    );

    create_table_without_asserts(db, create).await
}

pub async fn create_table_without_asserts(
    db: &DbConn,
    create: &TableCreateStatement,
) -> Result<ExecResult, DbErr> {
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
    db.execute(builder.build(create)).await
}
