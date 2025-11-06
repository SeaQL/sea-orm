pub mod staff {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "staff")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        pub reports_to_id: Option<i32>,
        #[sea_orm(
            self_ref,
            relation_enum = "ReportsTo",
            from = "reports_to_id",
            to = "id"
        )]
        pub reports_to: HasOne<Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}
