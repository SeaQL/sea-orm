mod cursor;
mod delete;
mod execute;
mod insert;
mod paginator;
mod query;
mod select;
mod update;

pub use cursor::*;
pub use delete::*;
pub use execute::*;
pub use insert::*;
pub use paginator::*;
pub use query::*;
pub use select::*;
pub use update::*;

use sea_orm::{
    entity::*,
    query::*,
    tests_cfg::cake::{self, Entity as Cake},
    DbBackend, DerivePartialModel, FromQueryResult,
};
use sea_query::{Expr, Func, SimpleExpr};
///
#[derive(DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "Cake")]
struct PartialCake {
    name: String,
    #[sea_orm(
        from_expr = r#"SimpleExpr::FunctionCall(Func::upper(Expr::col((Cake, cake::Column::Name))))"#
    )]
    name_upper: String,
}
///
assert_eq!(
    cake::Entity::find()
        .into_partial_model::<PartialCake>()
        .into_statement(DbBackend::Sqlite)
        .to_string(),
    r#"SELECT "cake"."name", UPPER("cake"."name") AS "name_upper" FROM "cake""#
);
