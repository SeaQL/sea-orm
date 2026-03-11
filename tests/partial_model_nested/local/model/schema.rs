use crate::common::setup::create_table;
use crate::local::{bakery, worker};
use sea_orm::{DatabaseConnection, DbConn, ExecResult, error::*, sea_query};
use sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, Index, Table};

pub async fn create_tables(db: &DatabaseConnection) -> Result<(), DbErr> {
    create_worker_table(db).await?;
    create_bakery_table(db).await?;
    Ok(())
}

pub async fn create_bakery_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = Table::create()
        .table(bakery::Entity)
        .col(
            ColumnDef::new(bakery::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(bakery::Column::Name).string().not_null())
        .col(
            ColumnDef::new(bakery::Column::ProfitMargin)
                .double()
                .not_null(),
        )
        .col(
            ColumnDef::new(bakery::Column::ManagerId)
                .integer()
                .not_null(),
        )
        .col(
            ColumnDef::new(bakery::Column::CashierId)
                .integer()
                .not_null(),
        )
        .foreign_key(
            ForeignKey::create()
                .name("fk-bakery-manager_id")
                .from(bakery::Entity, bakery::Column::ManagerId)
                .to(worker::Entity, worker::Column::Id),
        )
        .foreign_key(
            ForeignKey::create()
                .name("fk-bakery-cashier_id")
                .from(bakery::Entity, bakery::Column::CashierId)
                .to(worker::Entity, worker::Column::Id),
        )
        .to_owned();

    create_table(db, &stmt, bakery::Entity).await
}

pub async fn create_worker_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = Table::create()
        .table(worker::Entity)
        .col(
            ColumnDef::new(worker::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(worker::Column::Name).string().not_null())
        .to_owned();

    create_table(db, &stmt, worker::Entity).await
}
