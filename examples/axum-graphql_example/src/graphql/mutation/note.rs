use async_graphql::{Context, Object, Result};
use entity::async_graphql::{self, InputObject, SimpleObject};
use entity::note;
use entity::sea_orm::{ActiveModelTrait, Set};

use crate::db::Database;

// I normally separate the input types into separate files/modules, but this is just
// a quick example.

#[derive(InputObject)]
pub struct CreateNoteInput {
    pub title: String,
    pub text: String,
}

#[derive(SimpleObject)]
pub struct DeleteResult {
    pub success: bool,
    pub rows_affected: u64,
}

#[derive(Default)]
pub struct NoteMutation;

#[Object]
impl NoteMutation {
    pub async fn create_note(
        &self,
        ctx: &Context<'_>,
        input: CreateNoteInput,
    ) -> Result<note::Model> {
        let db = ctx.data::<Database>().unwrap();

        let note = note::ActiveModel {
            title: Set(input.title),
            text: Set(input.text),
            ..Default::default()
        };

        Ok(note.insert(db.get_connection()).await?)
    }

    pub async fn delete_note(&self, ctx: &Context<'_>, id: i32) -> Result<DeleteResult> {
        let db = ctx.data::<Database>().unwrap();

        let res = note::Entity::delete_by_id(id)
            .exec(db.get_connection())
            .await?;

        if res.rows_affected <= 1 {
            Ok(DeleteResult {
                success: true,
                rows_affected: res.rows_affected,
            })
        } else {
            unimplemented!()
        }
    }
}
