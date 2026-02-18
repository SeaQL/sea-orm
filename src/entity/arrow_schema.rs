/// Trait for Entities with Arrow integration
pub trait ArrowSchema {
    /// Get the Arrow schema for this Entity
    fn arrow_schema() -> arrow::datatypes::Schema;
}
