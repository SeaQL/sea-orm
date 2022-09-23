use async_graphql::{Context, Object, Result};
use entity::async_graphql::{self, InputObject, SimpleObject};
use entity::note;
use graphql_example_core::Mutation;

use crate::db::Database;

// I normally separate the input types into separate files/modules, but this is just
// a quick example.

#[derive(InputObject)]
pub struct CreateNoteInput {
    pub title: String,
    pub text: String,
}

impl CreateNoteInput {
    fn into_model_with_arbitrary_id(self) -> note::Model {
        note::Model {
            id: 0,
            title: self.title,
            text: self.text,
        }
    }
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
        let conn = db.get_connection();

        Ok(Mutation::create_note(conn, input.into_model_with_arbitrary_id()).await?)
    }

    pub async fn delete_note(&self, ctx: &Context<'_>, id: i32) -> Result<DeleteResult> {
        let db = ctx.data::<Database>().unwrap();
        let conn = db.get_connection();

        let res = Mutation::delete_note(conn, id)
            .await
            .expect("Cannot delete note");

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
