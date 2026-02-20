/// Trait for Entities with Arrow integration
pub trait ArrowSchema {
    /// Get the Arrow schema for this Entity
    fn arrow_schema() -> sea_orm_arrow::arrow::datatypes::Schema;
}
