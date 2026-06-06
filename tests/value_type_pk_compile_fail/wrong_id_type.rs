//! Passing a `PostId` to `user::Entity::find_by_id` must not compile.
//! `Id<user::Entity, _>` and `Id<post::Entity, _>` are type-distinct
//! despite having the same inner scalar.

use sea_orm::entity::prelude::*;

mod user {
    use sea_orm::entity::prelude::*;

    pub type UserId = sea_orm::Id<Entity, i32>;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "user")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: UserId,
        pub name: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

mod post {
    use sea_orm::entity::prelude::*;

    pub type PostId = sea_orm::Id<Entity, i32>;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "post")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: PostId,
        pub title: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

fn main() {
    // Cross-entity ID confusion, must not compile.
    let _ = user::Entity::find_by_id(post::PostId::new(1));
}
