pub trait ArrowSchema {
    fn arrow_schema() -> arrow::datatypes::Schema;
}