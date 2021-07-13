use sea_orm::{error::*, sea_query, DbConn, ExecResult};
use sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, Index, TableCreateStatement};

pub use super::super::bakery_chain::*;

async fn create_table(db: &DbConn, stmt: &TableCreateStatement) -> Result<ExecResult, DbErr> {
  let builder = db.get_schema_builder_backend();
  db.execute(builder.build(stmt)).await
}

pub async fn create_bakery_table(db: &DbConn) -> Result<ExecResult, DbErr> {
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
    .col(ColumnDef::new(bakery::Column::ProfitMargin).double())
    .to_owned();

  create_table(db, &stmt).await
}

pub async fn create_baker_table(db: &DbConn) -> Result<ExecResult, DbErr> {
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
    .col(ColumnDef::new(baker::Column::BakeryId).integer())
    .foreign_key(
      ForeignKey::create()
        .name("FK_baker_bakery")
        .from(baker::Entity, baker::Column::BakeryId)
        .to(bakery::Entity, bakery::Column::Id)
        .on_delete(ForeignKeyAction::Cascade)
        .on_update(ForeignKeyAction::Cascade),
    )
    .to_owned();

  create_table(db, &stmt).await
}

pub async fn create_customer_table(db: &DbConn) -> Result<ExecResult, DbErr> {
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
    .to_owned();

  create_table(db, &stmt).await
}

pub async fn create_order_table(db: &DbConn) -> Result<ExecResult, DbErr> {
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
        .from(order::Entity, order::Column::BakeryId)
        .to(bakery::Entity, bakery::Column::Id)
        .on_delete(ForeignKeyAction::Cascade)
        .on_update(ForeignKeyAction::Cascade),
    )
    .foreign_key(
      ForeignKey::create()
        .name("FK_order_customer")
        .from(order::Entity, order::Column::CustomerId)
        .to(customer::Entity, customer::Column::Id)
        .on_delete(ForeignKeyAction::Cascade)
        .on_update(ForeignKeyAction::Cascade),
    )
    .to_owned();

  create_table(db, &stmt).await
}

pub async fn create_lineitem_table(db: &DbConn) -> Result<ExecResult, DbErr> {
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
    .col(ColumnDef::new(lineitem::Column::Price).decimal())
    .col(ColumnDef::new(lineitem::Column::Quantity).integer())
    .col(
      ColumnDef::new(lineitem::Column::OrderId)
        .integer()
        .not_null(),
    )
    .col(
      ColumnDef::new(lineitem::Column::CakeId)
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
    .foreign_key(
      ForeignKey::create()
        .name("FK_lineitem_cake")
        .from(lineitem::Entity, lineitem::Column::CakeId)
        .to(cake::Entity, cake::Column::Id)
        .on_delete(ForeignKeyAction::Cascade)
        .on_update(ForeignKeyAction::Cascade),
    )
    .to_owned();

  create_table(db, &stmt).await
}

pub async fn create_cakes_bakers_table(db: &DbConn) -> Result<ExecResult, DbErr> {
  let stmt = sea_query::Table::create()
    .table(cakes_bakers::Entity)
    .if_not_exists()
    .col(
      ColumnDef::new(cakes_bakers::Column::CakeId)
        .integer()
        .not_null(),
    )
    .col(
      ColumnDef::new(cakes_bakers::Column::BakerId)
        .integer()
        .not_null(),
    )
    .primary_key(
      Index::create()
        .col(cakes_bakers::Column::CakeId)
        .col(cakes_bakers::Column::BakerId),
    )
    .to_owned();

  create_table(db, &stmt).await
}

pub async fn create_cake_table(db: &DbConn) -> Result<ExecResult, DbErr> {
  let stmt = sea_query::Table::create()
    .table(cake::Entity)
    .if_not_exists()
    .col(
      ColumnDef::new(cake::Column::Id)
        .integer()
        .not_null()
        .auto_increment()
        .primary_key(),
    )
    .col(ColumnDef::new(cake::Column::Name).string())
    .col(ColumnDef::new(cake::Column::Price).float())
    .col(ColumnDef::new(cake::Column::BakeryId).integer().not_null())
    .foreign_key(
      ForeignKey::create()
        .name("FK_cake_bakery")
        .from(cake::Entity, cake::Column::BakeryId)
        .to(bakery::Entity, bakery::Column::Id)
        .on_delete(ForeignKeyAction::Cascade)
        .on_update(ForeignKeyAction::Cascade),
    )
    .col(ColumnDef::new(cake::Column::GlutenFree).boolean())
    .to_owned();

  create_table(db, &stmt).await
}
