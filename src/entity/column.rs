pub use sea_query::ColumnType;
use sea_query::Iden;

pub trait Column: Iden {
    fn col_type(&self) -> ColumnType;
}
