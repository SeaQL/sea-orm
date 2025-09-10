use sea_orm::{
    ActiveModelTrait, DatabaseConnection, DeriveActiveEnum, EnumIter, Set, entity::prelude::*,
    sqlx::PgPool,
};
use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "example_enum")]
pub enum ExampleEnum {
    #[sea_orm(string_value = "first_variant")]
    FirstVariant,
    #[sea_orm(string_value = "second_variant")]
    SecondVariant,
}

#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "example_table")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub value: ExampleEnum,
    pub other_field: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(DerivePartialModel)]
#[sea_orm(entity = "Entity", from_query_result)]
pub struct PartialModel {
    pub id: i32,
    pub value: ExampleEnum,
}

#[tokio::main]
async fn main() {
    let connection_string = env::var("DATABASE_URL").unwrap();
    let pool = PgPool::connect(&connection_string).await.unwrap();
    let db = DatabaseConnection::from(pool);
    let _ = ActiveModel {
        value: Set(ExampleEnum::FirstVariant),
        other_field: Set(1),
        ..Default::default()
    }
    .insert(&db)
    .await
    .unwrap();
    let _: PartialModel = Entity::find_by_id(1)
        .into_partial_model()
        .one(&db)
        .await
        .unwrap()
        .unwrap();
}
