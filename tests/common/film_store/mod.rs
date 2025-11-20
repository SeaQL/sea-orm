pub mod film {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "film")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(unique)]
        pub title: String,
        #[sea_orm(has_many, via = "film_actor")]
        pub actors: HasMany<super::actor::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod actor {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "actor")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(unique)]
        pub name: String,
        #[sea_orm(has_many, via = "film_actor")]
        pub films: HasMany<super::film::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod film_actor {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "film_actor")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(unique_key = "film_actor")]
        pub film_id: i32,
        #[sea_orm(unique_key = "film_actor")]
        pub actor_id: i32,
        #[sea_orm(belongs_to, from = "film_id", to = "id")]
        pub film: HasOne<super::film::Entity>,
        #[sea_orm(belongs_to, from = "actor_id", to = "id")]
        pub actor: HasOne<super::actor::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

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
