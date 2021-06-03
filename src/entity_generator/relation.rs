#[derive(Clone, Debug)]
pub struct RelationSpec {
    name: String,
    ref_table: String,
    columns: Vec<String>,
    ref_columns: Vec<String>,
}
