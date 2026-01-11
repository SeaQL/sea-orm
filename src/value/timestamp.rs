macro_rules! impl_timestamp {
    ($ty:ident, $inner:ident, $from:ident, $to:ident) => {
        impl std::convert::From<$inner> for $ty {
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }

        impl std::convert::From<$ty> for sea_orm::Value {
            fn from(source: $ty) -> Self {
                $to(source).into()
            }
        }

        impl sea_orm::TryGetable for $ty {
            fn try_get_by<I: sea_orm::ColIdx>(
                res: &sea_orm::QueryResult,
                idx: I,
            ) -> std::result::Result<Self, TryGetError> {
                let ts = <i64 as sea_orm::TryGetable>::try_get_by(res, idx)?;
                $from(ts).ok_or(TryGetError::DbErr(DbErr::Type(
                    "Failed to convert i64 to timestamp".to_owned(),
                )))
            }
        }

        impl sea_orm::sea_query::ValueType for $ty {
            fn try_from(
                v: sea_orm::Value,
            ) -> std::result::Result<Self, sea_orm::sea_query::ValueTypeErr> {
                let ts = <i64 as sea_orm::sea_query::ValueType>::try_from(v)?;
                $from(ts).ok_or(sea_orm::sea_query::ValueTypeErr)
            }

            fn type_name() -> std::string::String {
                stringify!($ty).to_owned()
            }

            fn array_type() -> sea_orm::sea_query::ArrayType {
                <i64 as sea_orm::sea_query::ValueType>::array_type()
            }

            fn column_type() -> sea_orm::sea_query::ColumnType {
                <i64 as sea_orm::sea_query::ValueType>::column_type()
            }
        }

        impl sea_orm::sea_query::Nullable for $ty {
            fn null() -> sea_orm::Value {
                <i64 as sea_orm::sea_query::Nullable>::null()
            }
        }

        impl sea_orm::IntoActiveValue<$ty> for $ty {
            fn into_active_value(self) -> sea_orm::ActiveValue<$ty> {
                sea_orm::ActiveValue::Set(self)
            }
        }

        impl Deref for $ty {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl DerefMut for $ty {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl PartialEq<$inner> for $ty {
            fn eq(&self, other: &$inner) -> bool {
                self.0.eq(other)
            }
        }
    };
}

pub(super) use impl_timestamp;
