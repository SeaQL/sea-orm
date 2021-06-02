use crate::{ColumnSpec, RelationSpec};

pub struct EntitySpec {
    table_name: String,
    columns: Vec<ColumnSpec>,
    relations: Vec<RelationSpec>,
}
