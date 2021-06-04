use sea_orm::RelationType;

#[derive(Clone, Debug)]
pub struct Relation {
    pub(crate) ref_table: String,
    pub(crate) columns: Vec<String>,
    pub(crate) ref_columns: Vec<String>,
    pub(crate) rel_type: RelationType,
}
