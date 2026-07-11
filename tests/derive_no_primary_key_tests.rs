mod event_log {
    use sea_orm::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "event_log")]
    pub struct Model {
        pub kind: String,
        pub payload: String,
        pub level: i32,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

#[test]
fn no_primary_key_uses_fake_primary_key() {
    use sea_orm::{Iterable, PrimaryKeyTrait};

    let pks: Vec<_> = event_log::PrimaryKey::iter()
        .map(|pk| format!("{:?}", pk))
        .collect();
    assert_eq!(pks, vec!["FakePrimaryKey"]);

    assert!(!event_log::PrimaryKey::auto_increment());
}

#[test]
fn no_primary_key_custom_writes() {
    use sea_orm::{ColumnTrait, DbBackend, EntityTrait, QueryFilter, QueryTrait};

    let stmt = event_log::Entity::update_many()
        .col_expr(event_log::Column::Level, sea_orm::sea_query::Expr::value(0))
        .filter(event_log::Column::Kind.eq("error"))
        .build(DbBackend::Postgres)
        .to_string();
    assert_eq!(
        stmt,
        r#"UPDATE "event_log" SET "level" = 0 WHERE "event_log"."kind" = 'error'"#
    );

    let stmt = event_log::Entity::delete_many()
        .filter(event_log::Column::Level.lt(1))
        .build(DbBackend::Postgres)
        .to_string();
    assert_eq!(
        stmt,
        r#"DELETE FROM "event_log" WHERE "event_log"."level" < 1"#
    );
}
