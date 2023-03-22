use anyhow::Result;
use sea_orm::{ConnectionTrait, Database, EntityTrait, IntoActiveModel, Schema};

mod entity;

use entity::*;

#[tokio::main]
async fn main() -> Result<()> {
    let db = Database::connect("sqlite::memory:").await.unwrap();

    let builder = db.get_database_backend();
    let schema = Schema::new(builder);
    let stmt = schema.create_table_from_entity(Entity);
    db.execute(builder.build(&stmt)).await?;

    let model = Model {
        id: 100,
        name: "Hello".to_owned(),
    };

    let res = Entity::insert(model.clone().into_active_model())
        .exec(&db)
        .await?;

    assert_eq!(Entity::find().one(&db).await?, Some(model.clone()));
    assert_eq!(res.last_insert_id, model.id);

    let model = Model {
        id: -10,
        name: "World".to_owned(),
    };

    let res = Entity::insert(model.clone().into_active_model())
        .exec(&db)
        .await?;

    assert_eq!(Entity::find().one(&db).await?, Some(model.clone()));
    assert_eq!(res.last_insert_id, model.id);

    Ok(())
}
