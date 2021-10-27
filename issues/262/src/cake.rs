use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "cake")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub md5hash: String,
    pub md5_hash: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_transform_1() {
        assert_eq!(Column::Md5hash.to_string().as_str(), "md5hash");
        assert_eq!(Column::Md5Hash.to_string().as_str(), "md5_hash");
    }
}
