use sea_orm::prelude::*;
use sea_orm::Iden;
use sea_orm::Iterable;
use sea_orm_macros::DeriveEntityModel;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user")]
#[sea_orm(column_name_case = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key)]
    id: i32,
    username: String,
    first_name: String,
    middle_name: String,
    #[sea_orm(column_name = "lAsTnAmE")]
    last_name: String,
    orders_count: i32,
    #[sea_orm(column_name_case = "camelCase")]
    camel_case: String,
    #[sea_orm(column_name_case = "kebab-case")]
    kebab_case: String,
    #[sea_orm(column_name_case = "mixed_case")]
    mixed_case: String,
    #[sea_orm(column_name_case = "SCREAMING_SNAKE_CASE")]
    screaming_snake_case: String,
    #[sea_orm(column_name_case = "snake_case")]
    snake_case: String,
    #[sea_orm(column_name_case = "title_case")]
    title_case: String,
    #[sea_orm(column_name_case = "UPPERCASE")]
    upper_case: String,
    #[sea_orm(column_name_case = "lowercase")]
    lowercase: String,
    #[sea_orm(column_name_case = "SCREAMING-KEBAB-CASE")]
    screaming_kebab_case: String,
    #[sea_orm(column_name_case = "PascalCase")]
    pascal_case: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[test]
fn test_column_names() {
    let columns: Vec<String> = Column::iter().map(|item| item.to_string()).collect();

    assert_eq!(
        columns,
        vec![
            "id",
            "username",
            "firstName",
            "middleName",
            "lAsTnAmE",
            "ordersCount",
            "camelCase",
            "kebab-case",
            "mixedCase",
            "SCREAMING_SNAKE_CASE",
            "snake_case",
            "Title Case",
            "UPPERCASE",
            "lowercase",
            "SCREAMING-KEBAB-CASE",
            "PascalCase",
        ]
    );
}
