/// Apache Arrow schema for an entity.
///
/// Derived via `#[derive(DeriveArrowSchema)]`, this lets you exchange model
/// values with Arrow `RecordBatch`es — see
/// [`ActiveModelTrait::from_arrow`](crate::ActiveModelTrait::from_arrow) /
/// [`to_arrow`](crate::ActiveModelTrait::to_arrow). Requires the
/// `with-arrow` feature.
pub trait ArrowSchema {
    /// Arrow schema matching this entity's columns.
    fn arrow_schema() -> sea_orm_arrow::arrow::datatypes::Schema;
}
