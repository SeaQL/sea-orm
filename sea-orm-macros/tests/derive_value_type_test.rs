use sea_orm_macros::DeriveActiveEnum;

#[test]
fn when_user_import_nothing_macro_still_works_test() {
    #[derive(sea_orm::DeriveValueType)]
    struct MyString(String);
}

#[test]
fn when_user_alias_result_macro_still_works_test() {
    #[allow(dead_code)]
    type Result<T> = std::result::Result<T, ()>;
    #[derive(sea_orm::DeriveValueType)]
    struct MyString(String);
}

#[derive(DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "test_enum",
    rename_all = "camelCase"
)]
pub enum TestEnum {
    Variant,
    #[sea_orm(rename = "PascalCase")]
    AnotherVariant,
}
