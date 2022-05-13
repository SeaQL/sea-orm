use async_graphql::{Context, Object, Result};
use entity::{async_graphql, note};
use sea_orm::EntityTrait;

use crate::db::Database;

#[derive(Default)]
pub struct NoteQuery;

#[Object]
impl NoteQuery {
    async fn get_notes(&self, ctx: &Context<'_>) -> Result<Vec<note::Model>> {
        let db = ctx.data::<Database>().unwrap();

        Ok(note::Entity::find()
            .all(db.get_connection())
            .await
            .map_err(|e| e.to_string())?)
    }

    async fn get_note_by_id(&self, ctx: &Context<'_>, id: i32) -> Result<Option<note::Model>> {
        let db = ctx.data::<Database>().unwrap();

        Ok(note::Entity::find_by_id(id)
            .one(db.get_connection())
            .await
            .map_err(|e| e.to_string())?)
    }
}
