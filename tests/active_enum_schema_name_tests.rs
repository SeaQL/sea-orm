use sea_orm::DbBackend;

use common::features::Tea;
#[cfg(feature = "sqlx-postgres")]
use sea_orm::{QueryFilter, QueryTrait, entity::*};

#[path = "./common/mod.rs"]
pub mod common;

#[derive(Debug, Clone, PartialEq, Eq, sea_orm::EnumIter, sea_orm::DeriveActiveEnum)]
#[sea_orm(
    rs_type = "Enum",
    db_type = "Enum",
    enum_name = "mood",
    schema_name = "my_schema"
)]
enum Mood {
    #[sea_orm(string_value = "Happy")]
    Happy,
    #[sea_orm(string_value = "Sad")]
    Sad,
}

#[derive(Debug, Clone, PartialEq, Eq, sea_orm::EnumIter, sea_orm::DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "priority",
    schema_name = "my_schema"
)]
enum Priority {
    #[sea_orm(string_value = "Low")]
    Low,
    #[sea_orm(string_value = "High")]
    High,
}

mod schema_enum {
    use super::{Mood, Priority};
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[cfg_attr(feature = "sqlx-postgres", sea_orm(schema_name = "my_schema"))]
    #[sea_orm(table_name = "schema_enum")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub mood: Option<Mood>,
        pub priority: Option<Priority>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

#[test]
fn create_enum_from_active_enum_with_schema_name() {
    use sea_orm::{Schema, Statement};

    let db_postgres = DbBackend::Postgres;
    let schema = Schema::new(db_postgres);

    assert_eq!(
        db_postgres.build(&schema.create_enum_from_active_enum::<Mood>().unwrap()),
        Statement::from_string(
            db_postgres,
            r#"CREATE TYPE "my_schema"."mood" AS ENUM ('Happy', 'Sad')"#.to_owned()
        )
    );

    assert_eq!(
        db_postgres.build(&schema.create_enum_from_active_enum::<Priority>().unwrap()),
        Statement::from_string(
            db_postgres,
            r#"CREATE TYPE "my_schema"."priority" AS ENUM ('Low', 'High')"#.to_owned()
        )
    );
}

#[test]
fn active_enum_schema_name_returns_correct_value() {
    use sea_orm::ActiveEnum as ActiveEnumTrait2;

    assert_eq!(Mood::schema_name(), Some("my_schema"));
    assert_eq!(Tea::schema_name(), None);
}

#[test]
fn create_enum_without_schema_name_unchanged() {
    use sea_orm::{Schema, Statement};

    let db_postgres = DbBackend::Postgres;
    let schema = Schema::new(db_postgres);

    assert_eq!(
        db_postgres.build(&schema.create_enum_from_active_enum::<Tea>().unwrap()),
        Statement::from_string(
            db_postgres,
            r#"CREATE TYPE "tea" AS ENUM ('EverydayTea', 'BreakfastTea', 'AfternoonTea')"#
                .to_owned()
        )
    );
}

#[cfg(feature = "sqlx-postgres")]
#[test]
fn schema_enum_find_select_sql() {
    let select = schema_enum::Entity::find();

    assert_eq!(
        select.build(DbBackend::Postgres).to_string(),
        [
            r#"SELECT "schema_enum"."id","#,
            r#"CAST("schema_enum"."mood" AS "text"),"#,
            r#"CAST("schema_enum"."priority" AS "text")"#,
            r#"FROM "my_schema"."schema_enum""#,
        ]
        .join(" ")
    );
}

#[cfg(feature = "sqlx-postgres")]
#[test]
fn schema_enum_filter_is_in_sql() {
    let select = schema_enum::Entity::find()
        .filter(schema_enum::Column::Mood.is_in([Mood::Happy, Mood::Sad]));

    assert_eq!(
        select.build(DbBackend::Postgres).to_string(),
        [
            r#"SELECT "schema_enum"."id","#,
            r#"CAST("schema_enum"."mood" AS "text"),"#,
            r#"CAST("schema_enum"."priority" AS "text")"#,
            r#"FROM "my_schema"."schema_enum""#,
            r#"WHERE "schema_enum"."mood" IN ('Happy'::"my_schema"."mood", 'Sad'::"my_schema"."mood")"#,
        ]
        .join(" ")
    );
}

#[cfg(feature = "sqlx-postgres")]
#[test]
fn schema_enum_filter_eq_sql() {
    let select = schema_enum::Entity::find().filter(schema_enum::Column::Mood.eq(Mood::Happy));

    assert_eq!(
        select.build(DbBackend::Postgres).to_string(),
        [
            r#"SELECT "schema_enum"."id","#,
            r#"CAST("schema_enum"."mood" AS "text"),"#,
            r#"CAST("schema_enum"."priority" AS "text")"#,
            r#"FROM "my_schema"."schema_enum""#,
            r#"WHERE "schema_enum"."mood" = 'Happy'::"my_schema"."mood""#,
        ]
        .join(" ")
    );
}

#[test]
fn schema_enum_create_type_from_active_enum() {
    use sea_orm::{Schema, Statement};

    let db_postgres = DbBackend::Postgres;
    let schema = Schema::new(db_postgres);

    assert_eq!(
        db_postgres.build(&schema.create_enum_from_active_enum::<Priority>().unwrap()),
        Statement::from_string(
            db_postgres,
            r#"CREATE TYPE "my_schema"."priority" AS ENUM ('Low', 'High')"#.to_owned()
        )
    );
}
