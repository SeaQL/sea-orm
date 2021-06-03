use sea_query::ColumnType;

#[derive(Clone, Debug)]
pub struct ColumnSpec {
    name: String,
    rs_type: String,
    col_type: ColumnType,
    is_primary_key: bool,
}
