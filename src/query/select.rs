use crate::{ColumnTrait, EntityTrait, Iterable, QueryFilter, QueryOrder, QuerySelect, QueryTrait};
use core::fmt::Debug;
use core::marker::PhantomData;
pub use sea_query::JoinType;
use sea_query::{Alias, DynIden, Expr, Func, IntoColumnRef, SeaRc, SelectStatement, SimpleExpr};

#[derive(Clone, Debug)]
pub struct Select<E>
where
    E: EntityTrait,
{
    pub(crate) query: SelectStatement,
    pub(crate) entity: PhantomData<E>,
}

#[derive(Clone, Debug)]
pub struct SelectTwo<E, F>
where
    E: EntityTrait,
    F: EntityTrait,
{
    pub(crate) query: SelectStatement,
    pub(crate) entity: PhantomData<(E, F)>,
}

#[derive(Clone, Debug)]
pub struct SelectTwoMany<E, F>
where
    E: EntityTrait,
    F: EntityTrait,
{
    pub(crate) query: SelectStatement,
    pub(crate) entity: PhantomData<(E, F)>,
}

pub trait IntoSimpleExpr {
    fn into_simple_expr(self) -> SimpleExpr;
}

macro_rules! impl_trait {
    ( $trait: ident ) => {
        impl<E> $trait for Select<E>
        where
            E: EntityTrait,
        {
            type QueryStatement = SelectStatement;

            fn query(&mut self) -> &mut SelectStatement {
                &mut self.query
            }
        }

        impl<E, F> $trait for SelectTwo<E, F>
        where
            E: EntityTrait,
            F: EntityTrait,
        {
            type QueryStatement = SelectStatement;

            fn query(&mut self) -> &mut SelectStatement {
                &mut self.query
            }
        }

        impl<E, F> $trait for SelectTwoMany<E, F>
        where
            E: EntityTrait,
            F: EntityTrait,
        {
            type QueryStatement = SelectStatement;

            fn query(&mut self) -> &mut SelectStatement {
                &mut self.query
            }
        }
    };
}

impl_trait!(QuerySelect);
impl_trait!(QueryFilter);
impl_trait!(QueryOrder);

impl<C> IntoSimpleExpr for C
where
    C: ColumnTrait,
{
    fn into_simple_expr(self) -> SimpleExpr {
        SimpleExpr::Column(self.as_column_ref().into_column_ref())
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
        let table = SeaRc::new(E::default()) as DynIden;
        E::Column::iter()
            .map(|col| {
                let col_def = col.def();
                let enum_name = col_def.get_column_type().get_enum_name();
                let col_expr = Expr::tbl(table.clone(), col);
                let col_expr = if let Some(_) = enum_name {
                    Func::cast_expr_as(col_expr, Alias::new("text"))
                } else {
                    col_expr.into()
                };
                col_expr
            })
            .collect()
    }

    fn prepare_from(mut self) -> Self {
        self.query.from(E::default().table_ref());
        self
    }
}

impl<E> QueryTrait for Select<E>
where
    E: EntityTrait,
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

macro_rules! select_two {
    ( $selector: ident ) => {
        impl<E, F> QueryTrait for $selector<E, F>
        where
            E: EntityTrait,
            F: EntityTrait,
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

select_two!(SelectTwo);
select_two!(SelectTwoMany);
