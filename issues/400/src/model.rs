use std::marker::PhantomData;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "model")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: AccountId<String>,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, PartialEq)]
pub struct AccountId<T>(Uuid, PhantomData<T>);

impl<T> AccountId<T> {
    pub fn new(id: Uuid) -> Self {
        AccountId(id, PhantomData)
    }
}

impl<T> From<AccountId<T>> for Uuid {
    fn from(account_id: AccountId<T>) -> Self {
        account_id.0
    }
}

impl<T> sea_orm::TryFromU64 for AccountId<T> {
    fn try_from_u64(_n: u64) -> Result<Self, sea_orm::DbErr> {
        Err(sea_orm::DbErr::Exec(format!(
            "{} cannot be converted from u64",
            stringify!(AccountId<T>)
        )))
    }
}

impl<T> From<AccountId<T>> for sea_orm::Value {
    fn from(source: AccountId<T>) -> Self {
        sea_orm::Value::Uuid(Some(Box::new(source.into())))
    }
}

impl<T> sea_orm::TryGetable for AccountId<T> {
    fn try_get(
        res: &sea_orm::QueryResult,
        pre: &str,
        col: &str,
    ) -> Result<Self, sea_orm::TryGetError> {
        let val: Uuid = res.try_get(pre, col).map_err(sea_orm::TryGetError::DbErr)?;
        Ok(AccountId::<T>::new(val))
    }
}

impl<T> sea_orm::sea_query::Nullable for AccountId<T> {
    fn null() -> sea_orm::Value {
        sea_orm::Value::Uuid(None)
    }
}

impl<T> sea_orm::sea_query::ValueType for AccountId<T> {
    fn try_from(v: sea_orm::Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
        match v {
            sea_orm::Value::Uuid(Some(x)) => Ok(AccountId::<T>::new(*x)),
            _ => Err(sea_orm::sea_query::ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(AccountId).to_owned()
    }

    fn column_type() -> sea_orm::sea_query::ColumnType {
        sea_orm::sea_query::ColumnType::Uuid
    }
}
