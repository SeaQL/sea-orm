pub struct RelationSpec {
    name: String,
    columns: Vec<String>,
    ref_table: String,
    ref_columns: Vec<String>,
}
