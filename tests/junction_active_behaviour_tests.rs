#![allow(unused_imports, dead_code)]

//! Regression test for <https://github.com/SeaQL/sea-orm/issues/3010>:
//! `ActiveModelBehavior` (before_save / after_save) must run for the junction
//! table rows when establishing a many-to-many relation through the builder.

mod common;

use crate::common::TestContext;
use pretty_assertions::assert_eq;
use sea_orm::{ConnectionTrait, DbErr, Set, TryIntoModel, entity::prelude::*, entity::*};

mod jpost {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "j_post")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub title: String,
        #[sea_orm(has_many, via = "jpost_tag")]
        pub tags: HasMany<super::jtag::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod jtag {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "j_tag")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        #[sea_orm(has_many, via = "jpost_tag")]
        pub posts: HasMany<super::jpost::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod jlog {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "j_log")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub action: String,
        pub post_id: i32,
        pub tag_id: i32,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

// Junction table whose hooks log to `j_log`, so we can observe that they ran.
mod jpost_tag {
    use super::jlog;
    use sea_orm::{ConnectionTrait, Set, TryIntoModel, entity::prelude::*};

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "j_post_tag")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub post_id: i32,
        #[sea_orm(primary_key, auto_increment = false)]
        pub tag_id: i32,
        #[sea_orm(belongs_to, from = "post_id", to = "id")]
        pub post: Option<super::jpost::Entity>,
        #[sea_orm(belongs_to, from = "tag_id", to = "id")]
        pub tag: Option<super::jtag::Entity>,
    }

    async fn log<C: ConnectionTrait>(
        db: &C,
        action: &str,
        post_id: i32,
        tag_id: i32,
    ) -> Result<(), DbErr> {
        jlog::ActiveModel {
            action: Set(action.to_owned()),
            post_id: Set(post_id),
            tag_id: Set(tag_id),
            ..Default::default()
        }
        .insert(db)
        .await?;
        Ok(())
    }

    #[async_trait::async_trait]
    impl ActiveModelBehavior for ActiveModel {
        async fn before_save<C: ConnectionTrait>(
            self,
            db: &C,
            _insert: bool,
        ) -> Result<Self, DbErr> {
            let m = self.clone().try_into_model()?;
            log(db, "before_save", m.post_id, m.tag_id).await?;
            Ok(self)
        }

        async fn after_save<C: ConnectionTrait>(
            model: Model,
            db: &C,
            _insert: bool,
        ) -> Result<Model, DbErr> {
            log(db, "after_save", model.post_id, model.tag_id).await?;
            Ok(model)
        }
    }
}

#[sea_orm_macros::test]
async fn junction_active_behaviour() -> Result<(), DbErr> {
    let ctx = TestContext::new("junction_active_behaviour_tests").await;
    let db = &ctx.db;

    db.get_schema_builder()
        .register(jpost::Entity)
        .register(jtag::Entity)
        .register(jpost_tag::Entity)
        .register(jlog::Entity)
        .apply(db)
        .await?;

    // Persist a post with two new tags through the junction.
    jpost::ActiveModel::builder()
        .set_title("hello")
        .add_tag(jtag::ActiveModel::builder().set_name("rust"))
        .add_tag(jtag::ActiveModel::builder().set_name("orm"))
        .save(db)
        .await?;

    // The junction's before_save and after_save ran for each of the two rows.
    let logs = jlog::Entity::find().all(db).await?;
    let before = logs.iter().filter(|l| l.action == "before_save").count();
    let after = logs.iter().filter(|l| l.action == "after_save").count();
    assert_eq!(before, 2, "before_save should run for each junction row");
    assert_eq!(after, 2, "after_save should run for each junction row");

    Ok(())
}
