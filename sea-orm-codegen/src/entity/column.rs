use sea_query::ColumnType;

#[derive(Clone, Debug)]
pub struct Column {
    pub(crate) name: String,
    pub(crate) rs_type: String,
    pub(crate) col_type: ColumnType,
}
