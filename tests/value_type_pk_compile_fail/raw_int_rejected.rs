//! Passing a raw `i32` to `find_by_id` must not compile when the PK is
//! `Id<user::Entity, i32>`. `Id<E, T>` deliberately does not impl
//! `From<T>`, so `i32: !Into<Id<user::Entity, i32>>`.

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

fn main() {
    // Raw `i32` not convertible into `Id<user::Entity, i32>`.
    let _ = user::Entity::find_by_id(1i32);
}
