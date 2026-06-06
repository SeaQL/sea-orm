//! Comparing two `Id<E, T>` of DIFFERENT entities must not compile,
//! even when the inner scalar type matches. `PartialEq` is impl'd as
//! `impl<E, T> PartialEq for Id<E, T>` (same E), so `Id<post::Entity, i32>`
//! and `Id<user::Entity, i32>` have no shared `PartialEq` impl.

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
    // Direct equality across entities, must not compile.
    let _ = user::UserId::new(7) == post::PostId::new(7);
}
