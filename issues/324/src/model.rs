use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "model")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: AccountId,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, PartialEq)]
pub struct AccountId(Uuid);

impl From<AccountId> for Uuid {
    fn from(account_id: AccountId) -> Self {
        account_id.0
    }
}

macro_rules! impl_try_from_u64_err {
    ($newtype: ident) => {
        impl sea_orm::TryFromU64 for $newtype {
            fn try_from_u64(_n: u64) -> Result<Self, sea_orm::DbErr> {
                Err(sea_orm::CannotConvertFromU64(stringify!($newtype)))
            }
        }
    };
}

macro_rules! into_sea_query_value {
    ($newtype: ident: Box($name: ident)) => {
        impl From<$newtype> for sea_orm::Value {
            fn from(source: $newtype) -> Self {
                sea_orm::Value::$name(Some(Box::new(source.into())))
            }
        }

        impl sea_orm::TryGetable for $newtype {
            fn try_get(
                res: &sea_orm::QueryResult,
                pre: &str,
                col: &str,
            ) -> Result<Self, sea_orm::TryGetError> {
                let val: $name = res.try_get(pre, col).map_err(sea_orm::TryGetError::DbErr)?;
                Ok($newtype(val))
            }
        }

        impl sea_orm::sea_query::Nullable for $newtype {
            fn null() -> sea_orm::Value {
                sea_orm::Value::$name(None)
            }
        }

        impl sea_orm::sea_query::ValueType for $newtype {
            fn try_from(v: sea_orm::Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
                match v {
                    sea_orm::Value::$name(Some(x)) => Ok($newtype(*x)),
                    _ => Err(sea_orm::sea_query::ValueTypeErr),
                }
            }

            fn type_name() -> String {
                stringify!($newtype).to_owned()
            }

            fn column_type() -> sea_orm::sea_query::ColumnType {
                sea_orm::sea_query::ColumnType::$name
            }
        }
    };
}

into_sea_query_value!(AccountId: Box(Uuid));
impl_try_from_u64_err!(AccountId);
