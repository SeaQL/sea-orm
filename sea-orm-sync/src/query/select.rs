use crate::{
    ColumnTrait, EntityTrait, Iterable, Order, PrimaryKeyToColumn, QueryFilter, QueryOrder,
    QuerySelect, QueryTrait,
};
use core::fmt::Debug;
use core::marker::PhantomData;
use sea_query::{FunctionCall, IntoColumnRef, SelectStatement, SimpleExpr};

/// A `SELECT` query against entity `E`. Returned by
/// [`EntityTrait::find`](crate::EntityTrait::find); chain filters, joins,
/// ordering, and projections onto it, then run it on a
/// [`ConnectionTrait`](crate::ConnectionTrait) with `.one(db)` /
/// `.all(db)` / `.stream(db)` / `.paginate(db, n)`.
#[derive(Clone, Debug)]
pub struct Select<E>
where
    E: EntityTrait,
{
    pub(crate) query: SelectStatement,
    pub(crate) entity: PhantomData<E>,
    pub(crate) linked_index: usize,
}

/// A `SELECT` joining two entities, yielding `(E::Model, Option<F::Model>)`
/// per row — the right side is `None` for outer-join rows with no match.
/// Returned by [`Select::find_also_related`] and similar helpers.
#[derive(Clone, Debug)]
pub struct SelectTwo<E, F>
where
    E: EntityTrait,
    F: EntityTrait,
{
    pub(crate) query: SelectStatement,
    pub(crate) entity: PhantomData<(E, F)>,
}

/// A `SELECT` joining two entities, with results grouped into
/// `(E::Model, Vec<F::Model>)` per left model. Returned by
/// [`Select::find_with_related`].
#[derive(Clone, Debug)]
pub struct SelectTwoMany<E, F>
where
    E: EntityTrait,
    F: EntityTrait,
{
    pub(crate) query: SelectStatement,
    pub(crate) entity: PhantomData<(E, F)>,
}

/// A `SELECT` joining two entities where both sides are required, yielding
/// `(E::Model, F::Model)` per row (no `Option`).
#[derive(Clone, Debug)]
pub struct SelectTwoRequired<E, F>
where
    E: EntityTrait,
    F: EntityTrait,
{
    pub(crate) query: SelectStatement,
    pub(crate) entity: PhantomData<(E, F)>,
}

/// Marker trait describing how 3+ tables are joined: from one centre entity
/// to several siblings ([`TopologyStar`]) or in a sequential chain
/// ([`TopologyChain`]).
pub trait Topology {}

/// Star join: a centre entity joined separately to each of the others.
#[derive(Debug, Clone)]
pub struct TopologyStar;

/// Chain join: each entity joined to the previous one in sequence.
#[derive(Debug, Clone)]
pub struct TopologyChain;

impl Topology for TopologyStar {}
impl Topology for TopologyChain {}

/// Three-way join select, yielding `(E::Model, Option<F::Model>, Option<G::Model>)`.
/// The `TOP` parameter is the join [`Topology`].
#[derive(Clone, Debug)]
pub struct SelectThree<E, F, G, TOP>
where
    E: EntityTrait,
    F: EntityTrait,
    G: EntityTrait,
    TOP: Topology,
{
    pub(crate) query: SelectStatement,
    pub(crate) entity: PhantomData<(E, F, G, TOP)>,
}

/// Like [`SelectThree`], but results are consolidated under the left model:
/// `(E::Model, Vec<F::Model>, Vec<G::Model>)`.
#[derive(Clone, Debug)]
pub struct SelectThreeMany<E, F, G, TOP>
where
    E: EntityTrait,
    F: EntityTrait,
    G: EntityTrait,
    TOP: Topology,
{
    pub(crate) query: SelectStatement,
    pub(crate) entity: PhantomData<(E, F, G, TOP)>,
}

/// Four-way join select.
#[derive(Clone, Debug)]
pub struct SelectFour<E, F, G, H, TOP>
where
    E: EntityTrait,
    F: EntityTrait,
    G: EntityTrait,
    H: EntityTrait,
    TOP: Topology,
{
    pub(crate) query: SelectStatement,
    pub(crate) entity: PhantomData<(E, F, G, H, TOP)>,
}

/// Like [`SelectFour`], but results are consolidated under the left model.
#[derive(Clone, Debug)]
pub struct SelectFourMany<E, F, G, H, TOP>
where
    E: EntityTrait,
    F: EntityTrait,
    G: EntityTrait,
    H: EntityTrait,
    TOP: Topology,
{
    pub(crate) query: SelectStatement,
    pub(crate) entity: PhantomData<(E, F, G, H, TOP)>,
}

/// Five-way join select.
#[derive(Clone, Debug)]
pub struct SelectFive<E, F, G, H, I, TOP>
where
    E: EntityTrait,
    F: EntityTrait,
    G: EntityTrait,
    H: EntityTrait,
    I: EntityTrait,
    TOP: Topology,
{
    pub(crate) query: SelectStatement,
    pub(crate) entity: PhantomData<(E, F, G, H, I, TOP)>,
}

/// Six-way join select.
#[derive(Clone, Debug)]
pub struct SelectSix<E, F, G, H, I, J, TOP>
where
    E: EntityTrait,
    F: EntityTrait,
    G: EntityTrait,
    H: EntityTrait,
    I: EntityTrait,
    J: EntityTrait,
    TOP: Topology,
{
    pub(crate) query: SelectStatement,
    pub(crate) entity: PhantomData<(E, F, G, H, I, J, TOP)>,
}

/// Conversion into a [`SimpleExpr`]. Implemented for entity columns so they
/// can be used anywhere a `sea_query` expression is expected.
pub trait IntoSimpleExpr {
    /// Build the [`SimpleExpr`].
    fn into_simple_expr(self) -> SimpleExpr;
}

/// Like [`IntoSimpleExpr`] but applies SeaORM's enum-to-text cast for
/// `ActiveEnum` columns appearing in a SELECT list.
pub trait ColumnAsExpr: IntoSimpleExpr {
    /// Build the [`SimpleExpr`], casting `ActiveEnum` columns to text;
    /// otherwise identical to [`IntoSimpleExpr::into_simple_expr`].
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

        impl<E, F> $trait for SelectTwoRequired<E, F>
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

impl_query_trait!(QuerySelect);
impl_query_trait!(QueryFilter);
impl_query_trait!(QueryOrder);

impl<C> ColumnAsExpr for C
where
    C: ColumnTrait,
{
    fn into_column_as_expr(self) -> SimpleExpr {
        self.select_as(self.as_column_ref().into_column_ref().into())
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

impl IntoSimpleExpr for SimpleExpr {
    fn into_simple_expr(self) -> SimpleExpr {
        self
    }
}

impl IntoSimpleExpr for FunctionCall {
    fn into_simple_expr(self) -> SimpleExpr {
        SimpleExpr::FunctionCall(self)
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
            linked_index: 0,
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

    /// Apply order by primary key to the query statement
    pub fn order_by_id_asc(self) -> Self {
        self.order_by_id(Order::Asc)
    }

    /// Apply order by primary key to the query statement
    pub fn order_by_id_desc(self) -> Self {
        self.order_by_id(Order::Desc)
    }

    /// Apply order by primary key to the query statement
    pub fn order_by_id(mut self, order: Order) -> Self {
        for key in E::PrimaryKey::iter() {
            let col = key.into_column();
            self.query
                .order_by_expr(col.into_simple_expr(), order.clone());
        }
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
select_two!(SelectTwoRequired);
