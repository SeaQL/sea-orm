use crate::{ColumnTrait, DbErr, IdenStatic, PrimaryKeyArity, PrimaryKeyToColumn, PrimaryKeyTrait};
use crate::{QueryResult, TryFromU64, TryGetError, TryGetableMany};
use sea_query::{FromValueTuple, IntoValueTuple, ValueTuple};
use std::marker::PhantomData;

const NEVER_PRIMARY_KEY_IDEN: &str = "__never_primary_key__";

/// A dummy PrimaryKey type for entities without a primary key (e.g. database views).
///
/// This is mainly intended for read-only usage; APIs that rely on primary keys
/// (e.g. `find_by_id`) will generally be unusable for such entities.
#[derive(Copy, Clone, Debug, Default)]
pub struct NeverPrimaryKey<C>(PhantomData<C>);

impl<C> sea_query::Iden for NeverPrimaryKey<C> {
    fn unquoted(&self) -> &str {
        NEVER_PRIMARY_KEY_IDEN
    }
}

impl<C> IdenStatic for NeverPrimaryKey<C>
where
    C: ColumnTrait,
{
    fn as_str(&self) -> &'static str {
        NEVER_PRIMARY_KEY_IDEN
    }
}

impl<C> strum::IntoEnumIterator for NeverPrimaryKey<C> {
    type Iterator = std::iter::Empty<Self>;

    fn iter() -> Self::Iterator {
        std::iter::empty()
    }
}

impl<C> PrimaryKeyToColumn for NeverPrimaryKey<C>
where
    C: ColumnTrait,
{
    type Column = C;

    fn into_column(self) -> Self::Column {
        unreachable!("NeverPrimaryKey has no columns")
    }

    fn from_column(_: Self::Column) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }
}

impl<C> PrimaryKeyTrait for NeverPrimaryKey<C>
where
    C: ColumnTrait,
{
    type ValueType = NeverPrimaryKeyValue;

    fn auto_increment() -> bool {
        false
    }
}

/// The ValueType for [`NeverPrimaryKey`].
///
/// This type should never be constructed in normal code; it exists only to satisfy
/// trait bounds for entities without primary keys.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NeverPrimaryKeyValue {
    _priv: (),
}

impl PrimaryKeyArity for NeverPrimaryKeyValue {
    const ARITY: usize = 0;
}

impl From<NeverPrimaryKeyValue> for ValueTuple {
    fn from(_: NeverPrimaryKeyValue) -> Self {
        ValueTuple::Many(Vec::new())
    }
}

impl FromValueTuple for NeverPrimaryKeyValue {
    fn from_value_tuple<I>(_: I) -> Self
    where
        I: IntoValueTuple,
    {
        Self { _priv: () }
    }
}

impl TryGetableMany for NeverPrimaryKeyValue {
    fn try_get_many(_res: &QueryResult, _pre: &str, _cols: &[String]) -> Result<Self, TryGetError> {
        Err(TryGetError::DbErr(DbErr::Custom(
            "Entity has no primary key".to_owned(),
        )))
    }

    fn try_get_many_by_index(_res: &QueryResult) -> Result<Self, TryGetError> {
        Err(TryGetError::DbErr(DbErr::Custom(
            "Entity has no primary key".to_owned(),
        )))
    }
}

impl TryFromU64 for NeverPrimaryKeyValue {
    fn try_from_u64(_: u64) -> Result<Self, DbErr> {
        Err(DbErr::ConvertFromU64("NeverPrimaryKeyValue"))
    }
}
