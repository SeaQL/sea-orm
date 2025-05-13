use crate::{ColumnTrait, EntityTrait, Iterable, QueryFilter, QueryOrder, QuerySelect, QueryTrait};
use core::fmt::Debug;
use core::marker::PhantomData;
use sea_query::{Expr, IntoColumnRef, SelectStatement, SimpleExpr};

/// Defines a structure to perform select operations
#[derive(Clone, Debug)]
pub struct Select<E>
where
    E: EntityTrait,
{
    pub(crate) query: SelectStatement,
    pub(crate) entity: PhantomData<E>,
}

macro_rules! def_select_struct {
    ( $struct:ident <$($generics:ident),+>, $num:ident ) => {
        #[doc = concat!("Defines a structure to perform a SELECT operation on ", stringify!($num), " Models")]
        #[derive(Clone, Debug)]
        pub struct $struct<$($generics),*>
        where
            $($generics: EntityTrait),*
        {
            pub(crate) query: SelectStatement,
            pub(crate) entity: PhantomData<($($generics),*)>,
        }
    }
}

def_select_struct!(SelectTwo<E, F>, two);
def_select_struct!(SelectTwoMany<E, F>, many);
def_select_struct!(SelectThree<E, F, G>, three);
def_select_struct!(SelectFour<E, F, G, H>, four);
def_select_struct!(SelectFive<E, F, G, H, I>, five);
def_select_struct!(SelectSix<E, F, G, H, I, J>, six);
def_select_struct!(SelectSeven<E, F, G, H, I, J, K>, seven);
def_select_struct!(SelectEight<E, F, G, H, I, J, K, L>, eight);
def_select_struct!(SelectNine<E, F, G, H, I, J, K, L, M>, nine);
def_select_struct!(SelectTen<E, F, G, H, I, J, K, L, M, N>, ten);

/// Performs a conversion to [SimpleExpr]
pub trait IntoSimpleExpr {
    /// Method to perform the conversion
    fn into_simple_expr(self) -> SimpleExpr;
}

/// Extending [IntoSimpleExpr] to support casting ActiveEnum as TEXT in select expression
pub trait ColumnAsExpr: IntoSimpleExpr {
    /// Casting ActiveEnum as TEXT in select expression,
    /// otherwise same as [IntoSimpleExpr::into_simple_expr]
    fn into_column_as_expr(self) -> SimpleExpr;
}

macro_rules! impl_query_trait {
    ( $struct:ident <$($generics:ident),+> ) => {
        impl<$($generics),*> QuerySelect for $struct<$($generics),*>
        where
            $($generics: EntityTrait),*
        {
            type QueryStatement = SelectStatement;

            fn query(&mut self) -> &mut SelectStatement {
                &mut self.query
            }
        }

        impl<$($generics),*> QueryFilter for $struct<$($generics),*>
        where
            $($generics: EntityTrait),*
        {
            type QueryStatement = SelectStatement;

            fn query(&mut self) -> &mut SelectStatement {
                &mut self.query
            }
        }

        impl<$($generics),*> QueryOrder for $struct<$($generics),*>
        where
            $($generics: EntityTrait),*
        {
            type QueryStatement = SelectStatement;

            fn query(&mut self) -> &mut SelectStatement {
                &mut self.query
            }
        }

        impl<$($generics),*> QueryTrait for $struct<$($generics),*>
        where
            $($generics: EntityTrait),*
        {
            type QueryStatement = SelectStatement;

            fn query(&mut self) -> &mut SelectStatement {
                &mut self.query
            }
            fn as_query(&self) -> &SelectStatement {
                &self.query
            }
            fn into_query(self) -> SelectStatement {
                self.query
            }
        }
    };
}

impl_query_trait!(Select<E>);
impl_query_trait!(SelectTwo<E, F>);
impl_query_trait!(SelectTwoMany<E, F>);
impl_query_trait!(SelectThree<E, F, G>);
impl_query_trait!(SelectFour<E, F, G, H>);
impl_query_trait!(SelectFive<E, F, G, H, I>);
impl_query_trait!(SelectSix<E, F, G, H, I, J>);
impl_query_trait!(SelectSeven<E, F, G, H, I, J, K>);
impl_query_trait!(SelectEight<E, F, G, H, I, J, K, L>);
impl_query_trait!(SelectNine<E, F, G, H, I, J, K, L, M>);
impl_query_trait!(SelectTen<E, F, G, H, I, J, K, L, M, N>);

impl<C> ColumnAsExpr for C
where
    C: ColumnTrait,
{
    fn into_column_as_expr(self) -> SimpleExpr {
        self.select_as(Expr::expr(self.as_column_ref().into_column_ref()))
    }
}

impl ColumnAsExpr for Expr {
    fn into_column_as_expr(self) -> SimpleExpr {
        self.into_simple_expr()
    }
}

impl ColumnAsExpr for SimpleExpr {
    fn into_column_as_expr(self) -> SimpleExpr {
        self.into_simple_expr()
    }
}

impl<C> IntoSimpleExpr for C
where
    C: ColumnTrait,
{
    fn into_simple_expr(self) -> SimpleExpr {
        SimpleExpr::Column(self.as_column_ref().into_column_ref())
    }
}

impl IntoSimpleExpr for Expr {
    fn into_simple_expr(self) -> SimpleExpr {
        self.into()
    }
}

impl IntoSimpleExpr for SimpleExpr {
    fn into_simple_expr(self) -> SimpleExpr {
        self
    }
}

impl<E> Select<E>
where
    E: EntityTrait,
{
    pub(crate) fn new() -> Self {
        Self {
            query: SelectStatement::new(),
            entity: PhantomData,
        }
        .prepare_select()
        .prepare_from()
    }

    fn prepare_select(mut self) -> Self {
        self.query.exprs(self.column_list());
        self
    }

    fn column_list(&self) -> Vec<SimpleExpr> {
        E::Column::iter()
            .map(|col| col.select_as(col.into_expr()))
            .collect()
    }

    fn prepare_from(mut self) -> Self {
        self.query.from(E::default().table_ref());
        self
    }
}
