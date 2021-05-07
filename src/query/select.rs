use crate::Statement;
use crate::{entity::*, RelationDef};
use core::fmt::Debug;
use core::marker::PhantomData;
pub use sea_query::JoinType;
use sea_query::{Expr, Iden, IntoIden, QueryBuilder, SelectStatement};
use std::rc::Rc;
use strum::IntoEnumIterator;

#[derive(Debug)]
pub struct Select<'s, E: 'static>
where
    E: Entity,
{
    select: SelectStatement,
    entity: PhantomData<&'s E>,
}

impl<E: 'static> Select<'_, E>
where
    E: Entity,
{
    pub(crate) fn new(_: E) -> Self {
        Self {
            select: SelectStatement::new(),
            entity: PhantomData,
        }
        .prepare_select()
        .prepare_from()
    }

    fn prepare_select(mut self) -> Self {
        let table = E::default().into_iden();
        let columns: Vec<(Rc<dyn Iden>, E::Column)> =
            E::Column::iter().map(|c| (Rc::clone(&table), c)).collect();
        self.select.columns(columns);
        self
    }

    fn prepare_from(mut self) -> Self {
        self.select.from(E::default().into_iden());
        self
    }

    fn prepare_join(mut self, join: JoinType, relation: RelationDef) -> Self {
        let own_tbl = E::default().into_iden();
        let to_tbl = &relation.to_tbl;
        let owner_keys = relation.from_col;
        let foreign_keys = relation.to_col;
        let condition = match (owner_keys, foreign_keys) {
            (Identity::Unary(o1), Identity::Unary(f1)) => {
                Expr::tbl(Rc::clone(&own_tbl), o1).equals(Rc::clone(to_tbl), f1)
            } // _ => panic!("Owner key and foreign key mismatch"),
        };
        self.select.join(join, Rc::clone(to_tbl), condition);
        self
    }

    pub fn left_join(self, relation: RelationDef) -> Self {
        self.prepare_join(JoinType::LeftJoin, relation)
    }

    pub fn right_join(self, relation: RelationDef) -> Self {
        self.prepare_join(JoinType::RightJoin, relation)
    }

    pub fn inner_join(self, relation: RelationDef) -> Self {
        self.prepare_join(JoinType::InnerJoin, relation)
    }

    pub fn query(&mut self) -> &mut SelectStatement {
        &mut self.select
    }

    pub fn as_query(&self) -> &SelectStatement {
        &self.select
    }

    pub fn into_query(self) -> SelectStatement {
        self.select
    }

    pub fn build<B>(&self, builder: B) -> Statement
    where
        B: QueryBuilder,
    {
        self.as_query().build(builder).into()
    }
}
