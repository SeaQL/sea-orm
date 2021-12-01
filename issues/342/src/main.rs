mod model;

use model::*;
use sea_orm::{sea_query::Expr, *};

pub fn main() {
    // The result SQL statement doesn't make any sense as I just want to reproduce the compile errors
    assert_eq!(
        Entity::find()
            .select_only()
            .column_as(Expr::value(true), Column::Name)
            .filter(
                Expr::col(Column::Id)
                    .concatenate(Expr::col(Column::Name))
                    .equals(Expr::value(1)),
            )
            .build(DbBackend::Postgres)
            .to_string()
            .as_str(),
        r#"SELECT TRUE AS "name" FROM "cake" WHERE "id" || "name" = 1"#
    )
}
