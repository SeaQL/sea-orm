use entity::note::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let seed_data = vec![
            ("First Note", "This is the first note."),
            ("Second Note", "This is another note."),
        ];

        for (title, text) in seed_data {
            let model = ActiveModel {
                title: Set(title.to_string()),
                text: Set(text.to_string()),
                ..Default::default()
            };
            model.insert(db).await?;
        }

        println!("Notes table seeded successfully.");
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let titles_to_delete = vec!["First Note", "Second Note"];
        Entity::delete_many()
            .filter(Column::Title.is_in(titles_to_delete))
            .exec(db)
            .await?;

        println!("Notes seeded data removed.");
        Ok(())
    }
}
