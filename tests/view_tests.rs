#![allow(unused)]
use sea_orm::{ConnectionTrait, DbBackend, EntityTrait, Statement, Value, query::QueryOrder};

use crate::common::TestContext;

mod common;

mod cake_view {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "cake_view", view)]
    pub struct Model {
        pub id: i32,
        pub name: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter)]
    pub enum Relation {}

    impl RelationTrait for Relation {
        fn def(&self) -> RelationDef {
            panic!("Views are read-only and do not define relations")
        }
    }
}

fn cake_table_ddl(backend: DbBackend) -> &'static str {
    match backend {
        DbBackend::Postgres => "CREATE TABLE cake (id SERIAL PRIMARY KEY, name VARCHAR NOT NULL);",
        DbBackend::MySql => {
            "CREATE TABLE cake (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(255) NOT NULL);"
        }
        DbBackend::Sqlite => {
            "CREATE TABLE cake (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL);"
        }
        _ => unreachable!("unsupported backend for this integration test"),
    }
}

fn cake_view_ddl(backend: DbBackend) -> (&'static str, &'static str) {
    match backend {
        DbBackend::Postgres | DbBackend::MySql => (
            "DROP VIEW IF EXISTS cake_view;",
            "CREATE VIEW cake_view AS SELECT id, name FROM cake;",
        ),
        DbBackend::Sqlite => (
            "DROP VIEW IF EXISTS cake_view;",
            "CREATE VIEW cake_view AS SELECT id, name FROM cake;",
        ),
        _ => unreachable!("unsupported backend for this integration test"),
    }
}

#[sea_orm_macros::test]
async fn view_entity_can_query_but_cannot_modify() {
    let ctx = TestContext::new("view_entity_can_query_but_cannot_modify").await;
    let backend = ctx.db.get_database_backend();

    ctx.db
        .execute_raw(Statement::from_string(
            backend,
            "DROP VIEW IF EXISTS cake_view;".to_owned(),
        ))
        .await
        .unwrap();
    ctx.db
        .execute_raw(Statement::from_string(
            backend,
            "DROP TABLE IF EXISTS cake;".to_owned(),
        ))
        .await
        .unwrap();
    ctx.db
        .execute_raw(Statement::from_string(
            backend,
            cake_table_ddl(backend).to_owned(),
        ))
        .await
        .unwrap();

    ctx.db
        .execute_raw(Statement::from_string(
            backend,
            "INSERT INTO cake (name) VALUES ('Cheesecake'), ('Chocolate');".to_owned(),
        ))
        .await
        .unwrap();

    let (drop_view, create_view) = cake_view_ddl(backend);
    ctx.db
        .execute_raw(Statement::from_string(backend, drop_view.to_owned()))
        .await
        .unwrap();
    ctx.db
        .execute_raw(Statement::from_string(backend, create_view.to_owned()))
        .await
        .unwrap();

    let rows = cake_view::Entity::find()
        .order_by_asc(cake_view::Column::Id)
        .all(&ctx.db)
        .await
        .unwrap();

    assert_eq!(
        rows,
        vec![
            cake_view::Model {
                id: 1,
                name: "Cheesecake".to_owned()
            },
            cake_view::Model {
                id: 2,
                name: "Chocolate".to_owned()
            }
        ]
    );

    type ViewActiveModel = <cake_view::Entity as EntityTrait>::ActiveModel;
    let mut am = ViewActiveModel::default();
    let err = sea_orm::ActiveModelTrait::try_set(
        &mut am,
        cake_view::Column::Name,
        Value::from("New Name"),
    )
    .unwrap_err();
    assert!(matches!(err, sea_orm::DbErr::Custom(_)));

    ctx.delete().await;
}
