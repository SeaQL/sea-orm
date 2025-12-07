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
    pub(crate) persistent: Option<bool>,
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
    pub(crate) persistent: Option<bool>,
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
    pub(crate) persistent: Option<bool>,
}

/// Defines a structure to perform a SELECT operation on two Models
#[derive(Clone, Debug)]
pub struct SelectThree<E, F, G>
where
    E: EntityTrait,
    F: EntityTrait,
    G: EntityTrait,
{
    pub(crate) query: SelectStatement,
    pub(crate) entity: PhantomData<(E, F, G)>,
    pub(crate) persistent: Option<bool>,
}

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

        impl<E, F, G> $trait for SelectThree<E, F, G>
        where
            E: EntityTrait,
            F: EntityTrait,
            G: EntityTrait,
        {
            type QueryStatement = SelectStatement;

            fn query(&mut self) -> &mut SelectStatement {
                &mut self.query
            }
        }
    };
}

impl_query_trait!(QuerySelect);
impl_query_trait!(QueryFilter);
impl_query_trait!(QueryOrder);

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
            persistent: None,
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

    /// Set whether to use prepared statement caching
    ///
    /// # Examples
    ///
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres).into_connection();
    /// #
    /// use sea_orm::QuerySelect;
    ///
    /// let _select = cake::Entity::find()
    ///     .persistent(false);
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn persistent(mut self, value: bool) -> Self {
        self.persistent = Some(value);
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
    fn build(&self, db_backend: crate::DbBackend) -> crate::Statement {
        let query_builder = db_backend.get_query_builder();
        let mut statement = crate::Statement::from_string_values_tuple(
            db_backend,
            self.as_query().build_any(query_builder.as_ref()),
        );
        statement.persistent = self.persistent;
        statement
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
            fn build(&self, db_backend: crate::DbBackend) -> crate::Statement {
                let query_builder = db_backend.get_query_builder();
                let mut statement = crate::Statement::from_string_values_tuple(
                    db_backend,
                    self.as_query().build_any(query_builder.as_ref()),
                );
                statement.persistent = self.persistent;
                statement
            }
        }
    };
}

select_two!(SelectTwo);
select_two!(SelectTwoMany);

impl<E, F, G> QueryTrait for SelectThree<E, F, G>
where
    E: EntityTrait,
    F: EntityTrait,
    G: EntityTrait,
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
    fn build(&self, db_backend: crate::DbBackend) -> crate::Statement {
        let query_builder = db_backend.get_query_builder();
        let mut statement = crate::Statement::from_string_values_tuple(
            db_backend,
            self.as_query().build_any(query_builder.as_ref()),
        );
        statement.persistent = self.persistent;
        statement
    }
}
