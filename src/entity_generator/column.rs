use sea_query::ColumnType;

pub struct ColumnSpec {
    snake_name: String,
    pascal_name: String,
    rs_type: String,
    col_type: ColumnType,
    is_primary_key: bool,
}
