#![deny(clippy::future_not_send)]

mod parent {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "parent")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(has_many)]
        pub children: HasMany<super::child::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod child {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "child")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub parent_id: Option<i32>,
        #[sea_orm(belongs_to, from = "parent_id", to = "id")]
        pub parent: BelongsTo<Option<super::parent::Entity>>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

#[test]
fn generated_active_model_ex_mutation_futures_are_send() {
    let _ = parent::ActiveModel::builder();
    let _ = child::ActiveModel::builder();
}
