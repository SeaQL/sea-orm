use sea_query::ColumnType;

#[derive(Clone, Debug)]
pub struct ColumnSpec {
    pub(crate) name: String,
    pub(crate) rs_type: String,
    pub(crate) col_type: ColumnType,
    pub(crate) is_primary_key: bool,
}
