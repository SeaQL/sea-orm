mod cake {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "cake")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(unique)]
        pub name: String,
        #[sea_orm(unique_key = "pair")]
        pub left: i32,
        #[sea_orm(unique_key = "pair")]
        pub right: f64,
        #[sea_orm(has_one)]
        pub fruit: HasOne<super::fruit::Entity>,
        // #[sea_orm(has_many, via = "cake_filling")]
        // pub fillings: HasMany<super::filling::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod fruit {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "fruit")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        pub cake_id: Option<i32> ,
        #[sea_orm(belongs_to, from = "cake_id", to = "id")]
        pub cake: HasOne<super::cake::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

fn main() {}
