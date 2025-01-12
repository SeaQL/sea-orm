mod thread;
mod time_builder;

use sea_orm::{
    sea_query::TableCreateStatement, ActiveValue, ConnectionTrait, Database, DatabaseConnection,
    EntityTrait, QueryOrder, QuerySelect, Schema,
};
use time_builder::TimeBuilder;

#[tokio::main]
async fn main() {
    let conn: DatabaseConnection = Database::connect("sqlite::memory:").await.unwrap();
    create_thread_table(&conn).await;

    let thread_1 = create_thread(&conn).await;
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let thread_2 = create_thread(&conn).await;
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let thread_3 = create_thread(&conn).await;

    // This is not working
    let result = thread::Entity::find()
        .cursor_by(thread::Column::Id)
        .order_by_desc(thread::Column::Id)
        .limit(10)
        .all(&conn)
        .await
        .unwrap();

    // // This is working
    // let result = thread::Entity::find()
    //     .order_by_desc(thread::Column::Id)
    //     .all(&conn)
    //     .await
    //     .unwrap();

    assert_eq!(result[0].id, thread_3.id);
    assert_eq!(result[1].id, thread_2.id);
    assert_eq!(result[2].id, thread_1.id);
}

async fn create_thread(conn: &DatabaseConnection) -> thread::Model {
    thread::Entity::insert(thread::ActiveModel {
        created_at: ActiveValue::Set(TimeBuilder::now().into()),
        ..Default::default()
    })
    .exec_with_returning(conn)
    .await
    .unwrap()
}

async fn create_thread_table(conn: &DatabaseConnection) {
    let schema = Schema::new(sea_orm::DbBackend::Sqlite);
    let stmt: TableCreateStatement = schema.create_table_from_entity(thread::Entity);

    conn.execute(sea_orm::DbBackend::Sqlite.build(&stmt))
        .await
        .unwrap();
}
