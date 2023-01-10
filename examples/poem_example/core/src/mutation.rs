use ::entity::{post, post::Entity as Post};
use sea_orm::*;

pub struct Mutation;

impl Mutation {
    pub async fn create_post(
        db: &DatabaseTransaction,
        form_data: post::Model,
    ) -> Result<post::ActiveModel, DbErr> {
        post::ActiveModel {
            title: Set(form_data.title.to_owned()),
            text: Set(form_data.text.to_owned()),
            ..Default::default()
        }
        .save(db)
        .await
    }

    pub async fn update_post_by_id(
        db: &DatabaseTransaction,
        id: i32,
        form_data: post::Model,
    ) -> Result<post::Model, DbErr> {
        let post: post::ActiveModel = Post::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find post.".to_owned()))
            .map(Into::into)?;

        post::ActiveModel {
            id: post.id,
            title: Set(form_data.title.to_owned()),
            text: Set(form_data.text.to_owned()),
        }
        .update(db)
        .await
    }

    pub async fn delete_post(db: &DatabaseTransaction, id: i32) -> Result<DeleteResult, DbErr> {
        let post: post::ActiveModel = Post::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find post.".to_owned()))
            .map(Into::into)?;

        post.delete(db).await
    }

    pub async fn delete_all_posts(db: &DbConn) -> Result<DeleteResult, DbErr> {
        Post::delete_many().exec(db).await
    }
}
