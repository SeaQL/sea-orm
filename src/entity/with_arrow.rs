use crate::DbErr;
use arrow::array::Array;
use sea_query::{ColumnType, Value};

pub use sea_orm_arrow::ArrowError;

impl From<ArrowError> for DbErr {
    fn from(e: ArrowError) -> Self {
        DbErr::Type(e.to_string())
    }
}

pub(crate) fn arrow_array_to_value(
    array: &dyn Array,
    col_type: &ColumnType,
    row: usize,
) -> Result<Value, DbErr> {
    sea_orm_arrow::arrow_array_to_value(array, col_type, row).map_err(Into::into)
}

#[cfg(all(feature = "with-chrono", feature = "with-time"))]
pub(crate) fn arrow_array_to_value_alt(
    array: &dyn Array,
    col_type: &ColumnType,
    row: usize,
) -> Result<Option<Value>, DbErr> {
    sea_orm_arrow::arrow_array_to_value_alt(array, col_type, row).map_err(Into::into)
}

pub(crate) fn is_datetime_column(col_type: &ColumnType) -> bool {
    sea_orm_arrow::is_datetime_column(col_type)
}

pub(crate) fn values_to_arrow_array(
    values: &[Option<Value>],
    data_type: &arrow::datatypes::DataType,
) -> Result<std::sync::Arc<dyn Array>, DbErr> {
    sea_orm_arrow::values_to_arrow_array(values, data_type).map_err(Into::into)
}
