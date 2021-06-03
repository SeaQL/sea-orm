use crate::{ColumnSpec, RelationSpec};

#[derive(Clone, Debug)]
pub struct EntitySpec {
    table_name: String,
    columns: Vec<ColumnSpec>,
    relations: Vec<RelationSpec>,
}
