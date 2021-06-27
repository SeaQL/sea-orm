use sea_orm::{sea_query, DbConn, ExecErr, ExecResult};
use sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, SqliteQueryBuilder};

pub mod bakery_chain;
mod setup;
pub use bakery_chain::*;

#[async_std::test]
// cargo test --test bakery_chain_tests -- --nocapture
async fn main() {
    let db: DbConn = setup::setup().await;
    setup_schema(&db).await;
}

async fn setup_schema(db: &DbConn) {
    assert!(create_bakery_table(db).await.is_ok());
    assert!(create_baker_table(db).await.is_ok());
    assert!(create_customer_table(db).await.is_ok());
    assert!(create_order_table(db).await.is_ok());
    assert!(create_lineitem_table(db).await.is_ok());
}

async fn create_bakery_table(db: &DbConn) -> Result<ExecResult, ExecErr> {
    let stmt = sea_query::Table::create()
        .table(bakery::Entity)
        .if_not_exists()
        .col(
            ColumnDef::new(bakery::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(bakery::Column::Name).string())
        .col(ColumnDef::new(bakery::Column::ProfitMargin).float())
        .build(SqliteQueryBuilder);

    db.execute(stmt.into()).await
}

async fn create_baker_table(db: &DbConn) -> Result<ExecResult, ExecErr> {
    let stmt = sea_query::Table::create()
        .table(baker::Entity)
        .if_not_exists()
        .col(
            ColumnDef::new(baker::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(baker::Column::Name).string())
        .col(ColumnDef::new(baker::Column::BakeryId).integer().not_null())
        .foreign_key(
            ForeignKey::create()
                .name("FK_baker_bakery")
                .from(baker::Entity, baker::Column::BakeryId)
                .to(bakery::Entity, bakery::Column::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade),
        )
        .build(SqliteQueryBuilder);

    db.execute(stmt.into()).await
}

async fn create_customer_table(db: &DbConn) -> Result<ExecResult, ExecErr> {
    let stmt = sea_query::Table::create()
        .table(customer::Entity)
        .if_not_exists()
        .col(
            ColumnDef::new(customer::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(customer::Column::Name).string())
        .col(ColumnDef::new(customer::Column::Notes).text())
        .build(SqliteQueryBuilder);

    db.execute(stmt.into()).await
}

async fn create_order_table(db: &DbConn) -> Result<ExecResult, ExecErr> {
    let stmt = sea_query::Table::create()
        .table(order::Entity)
        .if_not_exists()
        .col(
            ColumnDef::new(order::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(order::Column::Total).float())
        .col(ColumnDef::new(order::Column::BakeryId).integer().not_null())
        .col(
            ColumnDef::new(order::Column::CustomerId)
                .integer()
                .not_null(),
        )
        .col(
            ColumnDef::new(order::Column::PlacedAt)
                .date_time()
                .not_null(),
        )
        .foreign_key(
            ForeignKey::create()
                .name("FK_order_bakery")
                .from(order::Entity, baker::Column::BakeryId)
                .to(bakery::Entity, bakery::Column::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade),
        )
        .foreign_key(
            ForeignKey::create()
                .name("FK_order_customer")
                .from(order::Entity, baker::Column::BakeryId)
                .to(customer::Entity, customer::Column::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade),
        )
        .build(SqliteQueryBuilder);

    db.execute(stmt.into()).await
}

async fn create_lineitem_table(db: &DbConn) -> Result<ExecResult, ExecErr> {
    let stmt = sea_query::Table::create()
        .table(lineitem::Entity)
        .if_not_exists()
        .col(
            ColumnDef::new(lineitem::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(lineitem::Column::Price).float())
        .col(ColumnDef::new(lineitem::Column::Quantity).integer())
        .col(
            ColumnDef::new(lineitem::Column::OrderId)
                .integer()
                .not_null(),
        )
        .foreign_key(
            ForeignKey::create()
                .name("FK_lineitem_order")
                .from(lineitem::Entity, lineitem::Column::OrderId)
                .to(order::Entity, order::Column::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade),
        )
        .build(SqliteQueryBuilder);

    db.execute(stmt.into()).await
}
