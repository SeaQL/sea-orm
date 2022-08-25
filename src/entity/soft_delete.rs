use sea_query::{DynIden, Expr, Func, SimpleExpr};

/// A trait for the configuration of soft delete
pub trait SoftDeleteTrait {
    /// Specify the column for soft delete
    fn soft_delete_column() -> Option<DynIden>;

    /// Mark a row is being soft deleted by filling the soft delete column with this expression (value)
    fn soft_delete_expr() -> SimpleExpr {
        Func::current_timestamp()
    }

    /// Undo a soft delete by setting soft delete column to this expression (value)
    fn restore_soft_delete_expr() -> SimpleExpr {
        Expr::cust("NULL")
    }
}
