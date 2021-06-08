use crate::{Column, PrimaryKey, Relation};

#[derive(Clone, Debug)]
pub struct Entity {
    pub(crate) table_name: String,
    pub(crate) columns: Vec<Column>,
    pub(crate) relations: Vec<Relation>,
    pub(crate) primary_keys: Vec<PrimaryKey>,
}
