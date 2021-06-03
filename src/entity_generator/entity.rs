use crate::{ColumnSpec, RelationSpec};

#[derive(Clone, Debug)]
pub struct EntitySpec {
    pub(crate) table_name: String,
    pub(crate) columns: Vec<ColumnSpec>,
    pub(crate) relations: Vec<RelationSpec>,
}
