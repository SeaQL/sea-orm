use crate::{ColumnTrait, EntityTrait, Iterable, QueryHelper, Statement};
use core::fmt::Debug;
use core::marker::PhantomData;
pub use sea_query::JoinType;
use sea_query::{Iden, IntoColumnRef, IntoIden, QueryBuilder, SelectStatement, SimpleExpr};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Select<E, S>
where
    E: EntityTrait,
    S: SelectState,
{
    pub(crate) query: SelectStatement,
    pub(crate) entity: PhantomData<E>,
    pub(crate) state: PhantomData<S>,
}

#[derive(Clone, Debug)]
pub struct SelectTwo<E, F, S>
where
    E: EntityTrait,
    F: EntityTrait,
    S: SelectState,
{
    pub(crate) query: SelectStatement,
    pub(crate) entity: PhantomData<(E, F)>,
    pub(crate) state: PhantomData<S>,
}

pub trait SelectState {}

pub struct SelectStateEmpty;

pub struct SelectStateHasCondition;

pub trait IntoSimpleExpr {
    fn into_simple_expr(self) -> SimpleExpr;
}

impl SelectState for SelectStateEmpty {}

impl SelectState for SelectStateHasCondition {}

impl<E, S> QueryHelper for Select<E, S>
where
    E: EntityTrait,
    S: SelectState,
{
    fn query(&mut self) -> &mut SelectStatement {
        &mut self.query
    }
}

impl<E, F, S> QueryHelper for SelectTwo<E, F, S>
where
    E: EntityTrait,
    F: EntityTrait,
    S: SelectState,
{
    fn query(&mut self) -> &mut SelectStatement {
        &mut self.query
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

impl IntoSimpleExpr for SimpleExpr {
    fn into_simple_expr(self) -> SimpleExpr {
        self
    }
}

impl<E, S> Select<E, S>
where
    E: EntityTrait,
    S: SelectState,
{
    pub(crate) fn new() -> Self {
        Self {
            query: SelectStatement::new(),
            entity: PhantomData,
            state: PhantomData,
        }
        .prepare_select()
        .prepare_from()
    }

    fn prepare_select(mut self) -> Self {
        self.query.columns(self.column_list());
        self
    }

    fn column_list(&self) -> Vec<(Rc<dyn Iden>, E::Column)> {
        let table = Rc::new(E::default()) as Rc<dyn Iden>;
        E::Column::iter().map(|col| (table.clone(), col)).collect()
    }

    fn prepare_from(mut self) -> Self {
        self.query.from(E::default().into_iden());
        self
    }

    /// Get a mutable ref to the query builder
    pub fn query(&mut self) -> &mut SelectStatement {
        &mut self.query
    }

    /// Get an immutable ref to the query builder
    pub fn as_query(&self) -> &SelectStatement {
        &self.query
    }

    /// Take ownership of the query builder
    pub fn into_query(self) -> SelectStatement {
        self.query
    }

    /// Build the query as [`Statement`]
    pub fn build<B>(&self, builder: B) -> Statement
    where
        B: QueryBuilder,
    {
        self.as_query().build(builder).into()
    }
}

impl<E, F, S> SelectTwo<E, F, S>
where
    E: EntityTrait,
    F: EntityTrait,
    S: SelectState,
{
    /// Get a mutable ref to the query builder
    pub fn query(&mut self) -> &mut SelectStatement {
        &mut self.query
    }

    /// Get an immutable ref to the query builder
    pub fn as_query(&self) -> &SelectStatement {
        &self.query
    }

    /// Take ownership of the query builder
    pub fn into_query(self) -> SelectStatement {
        self.query
    }

    /// Build the query as [`Statement`]
    pub fn build<B>(&self, builder: B) -> Statement
    where
        B: QueryBuilder,
    {
        self.as_query().build(builder).into()
    }
}
