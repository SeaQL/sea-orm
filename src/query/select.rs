use crate::{ColumnTrait, EntityTrait, Iterable, QueryFilter, QueryOrder, QuerySelect, QueryTrait};
use core::fmt::Debug;
use core::marker::PhantomData;
pub use sea_query::JoinType;
use sea_query::{Alias, DynIden, Expr, IntoColumnRef, SeaRc, SelectStatement, SimpleExpr};

/// Defines a structure to perform select operations
#[derive(Clone, Debug)]
pub struct Select<E>
where
    E: EntityTrait,
{
    pub(crate) query: SelectStatement,
    pub(crate) entity: PhantomData<E>,
}

/// Defines a structure to perform a SELECT operation on two Models
#[derive(Clone, Debug)]
pub struct SelectTwo<E, F>
where
    E: EntityTrait,
    F: EntityTrait,
{
    pub(crate) query: SelectStatement,
    pub(crate) entity: PhantomData<(E, F)>,
}

/// Defines a structure to perform a SELECT operation on many Models
#[derive(Clone, Debug)]
pub struct SelectTwoMany<E, F>
where
    E: EntityTrait,
    F: EntityTrait,
{
    pub(crate) query: SelectStatement,
    pub(crate) entity: PhantomData<(E, F)>,
}

/// Performs a conversion to [SimpleExpr]
pub trait IntoSimpleExpr {
    /// Method to perform the conversion
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
        let text_type = SeaRc::new(Alias::new("text")) as DynIden;
        E::Column::iter()
            .map(|col| {
                let expr = Expr::tbl(table.clone(), col);
                let col_def = col.def();
                let col_type = col_def.get_column_type();
                match col_type.get_enum_name() {
                    Some(_) => expr.as_enum(text_type.clone()),
                    None => expr.into(),
                }
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
