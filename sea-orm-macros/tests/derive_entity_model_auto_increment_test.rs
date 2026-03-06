use sea_orm::entity::prelude::*;
use sea_orm_macros::DeriveEntityModel;

mod string_pk {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "string_pk")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

mod string_pk_set_true {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "string_pk")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = true)]
        pub id: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

#[test]
fn test_auto_increment_default_by_type() {
    assert!(!string_pk::PrimaryKey::auto_increment());
    assert!(string_pk_set_true::PrimaryKey::auto_increment());
}
