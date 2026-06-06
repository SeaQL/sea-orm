//! Self-ref junction tables (where 2+ FK columns point at the same parent)
//! get per-column "role wrapper" structs around the parent's PK alias.
//! This makes positional swaps (passing one role where the other is
//! expected) fail to compile.

use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;

mod user {
    use sea_orm::entity::prelude::*;

    pub type UserId = sea_orm::Id<Entity, i32>;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "user")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment)]
        pub id: UserId,
        pub name: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

mod user_follower {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, DeriveValueType)]
    #[sea_orm(try_from_u64)]
    pub struct UserFollowerPkUserId(pub super::user::UserId);

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, DeriveValueType)]
    #[sea_orm(try_from_u64)]
    pub struct UserFollowerPkFollowerId(pub super::user::UserId);

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "user_follower")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub user_id: UserFollowerPkUserId,
        #[sea_orm(primary_key, auto_increment = false)]
        pub follower_id: UserFollowerPkFollowerId,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

fn main() {
    let user = user::Model {
        id: user::UserId::new(1),
        name: "alice".into(),
    };
    let follower = user::Model {
        id: user::UserId::new(2),
        name: "bob".into(),
    };

    // Deliberately swapped roles. The role wrappers make these distinct
    // types: `UserFollowerPkUserId` and `UserFollowerPkFollowerId` are not
    // inter-convertible, so passing one where the other is expected fails.
    let _ = user_follower::ActiveModel {
        user_id: Set(user_follower::UserFollowerPkFollowerId(follower.id)), // wrong slot
        follower_id: Set(user_follower::UserFollowerPkUserId(user.id)),     // wrong slot
    };
}
