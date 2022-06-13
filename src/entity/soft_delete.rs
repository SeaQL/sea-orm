use sea_query::{DynIden, Func, SimpleExpr};

///
pub trait SoftDeleteTrait {
    /// Specify the column for soft delete
    fn soft_delete_column() -> Option<DynIden>;

    ///
    fn soft_delete_expr() -> SimpleExpr {
        Func::current_timestamp()
    }
}
