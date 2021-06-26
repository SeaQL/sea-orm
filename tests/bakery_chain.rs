use sea_orm::{sea_query, DbConn, ExecErr, ExecResult};
use sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, Iden, SqliteQueryBuilder};

mod setup;

#[derive(Iden)]
enum Bakery {
    Table,
    Id,
    Name,
    ProfitMargin,
}

#[derive(Iden)]
enum Baker {
    Table,
    Id,
    Name,
    BakeryId,
}

#[async_std::test]
// cargo test --test bakery -- --nocapture
async fn main() {
    let db: DbConn = setup::setup().await;
    setup_schema(&db).await;
}

async fn setup_schema(db: &DbConn) {
    assert!(create_bakery(db).await.is_ok());
    assert!(create_baker(db).await.is_ok());
}

async fn create_bakery(db: &DbConn) -> Result<ExecResult, ExecErr> {
    let stmt = sea_query::Table::create()
        .table(Bakery::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(Bakery::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(Bakery::Name).string())
        .col(ColumnDef::new(Bakery::ProfitMargin).float())
        .build(SqliteQueryBuilder);
    db.execute(stmt.into()).await
}

async fn create_baker(db: &DbConn) -> Result<ExecResult, ExecErr> {
    let stmt = sea_query::Table::create()
        .table(Baker::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(Baker::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(Baker::Name).string())
        // .foreign_key(
        //     ForeignKey::create()
        //         .name("FK_baker_bakery")
        //         .from(Baker::Table, Baker::BakeryId)
        //         .to(Bakery::Table, Bakery::Id)
        //         .on_delete(ForeignKeyAction::Cascade)
        //         .on_update(ForeignKeyAction::Cascade),
        // )
        .build(SqliteQueryBuilder);

    db.execute(stmt.clone().into()).await
}
