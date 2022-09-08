use ::entity::{note, note::Entity as Note};
use sea_orm::*;

pub struct Mutation;

impl Mutation {
    pub async fn create_note(db: &DbConn, form_data: note::Model) -> Result<note::Model, DbErr> {
        let active_model = note::ActiveModel {
            title: Set(form_data.title.to_owned()),
            text: Set(form_data.text.to_owned()),
            ..Default::default()
        };
        let res = Note::insert(active_model).exec(db).await?;

        Ok(note::Model {
            id: res.last_insert_id,
            ..form_data
        })
    }

    pub async fn update_note_by_id(
        db: &DbConn,
        id: i32,
        form_data: note::Model,
    ) -> Result<note::Model, DbErr> {
        let note: note::ActiveModel = if let Ok(Some(note)) = Note::find_by_id(id).one(db).await {
            note.into()
        } else {
            return Err(DbErr::Custom("Cannot find note.".to_owned()));
        };

        note::ActiveModel {
            id: note.id,
            title: Set(form_data.title.to_owned()),
            text: Set(form_data.text.to_owned()),
        }
        .update(db)
        .await
    }

    pub async fn delete_note(db: &DbConn, id: i32) -> Result<DeleteResult, DbErr> {
        let note: note::ActiveModel = if let Ok(Some(note)) = Note::find_by_id(id).one(db).await {
            note.into()
        } else {
            return Err(DbErr::Custom("Cannot find note.".to_owned()));
        };

        note.delete(db).await
    }

    pub async fn delete_all_notes(db: &DbConn) -> Result<DeleteResult, DbErr> {
        Note::delete_many().exec(db).await
    }
}
