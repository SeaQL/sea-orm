use sea_orm::{prelude::*, FromQueryResult};
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "someenum"
)]
pub enum SomeEnum {
    #[sea_orm(string_value = "foo")]
    Foo,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "some_table")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub status: Option<SomeEnum>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

mod tests {
    use sea_orm::{tests_cfg::cake, DbBackend, Insert, QuerySelect, QueryTrait, Select, SelectColumns};

    use super::*;

    #[test]
    fn insert_do_nothing_postgres() {
        let exp1 = Entity::find()
                .select_only()
                .column(Column::Status)
                .build(DbBackend::Postgres)
                .sql;
        
        let exp2 = Entity::find()
                .select_only()
                .column_as(Column::Status, "status")
                .build(DbBackend::Postgres)
                .sql;
            
        // This seems like it should match...?
        // // assert_eq!(exp1, exp2);
        // But it doesn't...
        assert_ne!(exp1, exp2);
        // The result is that select_as can't be used with an enum, because DecodeColumn expects Enums to be Text.
        // So it throws a mismatched_types error.
        // https://github.com/launchbadge/sqlx/blob/293c55ce896edefc797b6b819c8b27cd327382bb/sqlx-core/src/error.rs#L156
        // For example:
        // Error: DatabaseError(Query(SqlxError(ColumnDecode { index: "\"realestateproject_status\"", source: "mismatched types; Rust type `core::option::Option<alloc::string::String>` (as SQL type `TEXT`) is not compatible with SQL type `realestateprojectstatus`" })))

        // Workaround
        let exp1 = Entity::find()
            .select_only()
            .column(Column::Status)
            .build(DbBackend::Postgres)
            .sql;

        let exp2 = Entity::find()
            .select_only()
            .column_as(
                Expr::col(Column::Status)
                    .cast_as(sea_orm::sea_query::Alias::new("TEXT")),
                "status",
            )
            .build(DbBackend::Postgres)
            .sql;

        // This matches functionally, but not exactly. It's a workaround.
        // assert_eq!(exp1, exp2);

        // left: "SELECT CAST(\"some_table\".\"status\" AS text) FROM \"some_table\""
        // right: "SELECT CAST(\"status\" AS TEXT) AS \"status\" FROM \"some_table\""

        // It isn't clear to me WHY the type is being case to text and then back, rather than just being cast to a Rust enum.
        // The reasoning isn't documented. It seems like it should be possible to cast directly to the enum.
    }
}