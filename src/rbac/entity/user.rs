use crate as sea_orm;
use sea_orm::DeriveValueType;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, DeriveValueType)]
pub struct UserId(pub i64);
