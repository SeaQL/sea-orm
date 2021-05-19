use crate::{EntityTrait, Identity, IntoIdentity, QueryHelper, Select};
use core::marker::PhantomData;
use sea_query::{Iden, IntoIden, JoinType};
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Debug)]
pub enum RelationType {
    HasOne,
    HasMany,
}

pub trait RelationTrait: Debug + 'static {
    fn def(&self) -> RelationDef;
}

pub trait Related<R>
where
    R: EntityTrait,
{
    fn to() -> RelationDef;

    fn via() -> Option<RelationDef> {
        None
    }

    fn find_related() -> Select<R> {
        Select::<R>::new().join_join_rev(JoinType::InnerJoin, Self::to(), Self::via())
    }
}

pub struct RelationDef {
    pub rel_type: RelationType,
    pub from_tbl: Rc<dyn Iden>,
    pub to_tbl: Rc<dyn Iden>,
    pub from_col: Identity,
    pub to_col: Identity,
}

pub struct RelationBuilder<E, R>
where
    E: EntityTrait,
    R: EntityTrait,
{
    entities: PhantomData<(E, R)>,
    rel_type: RelationType,
    from_tbl: Rc<dyn Iden>,
    to_tbl: Rc<dyn Iden>,
    from_col: Option<Identity>,
    to_col: Option<Identity>,
}

impl RelationDef {
    /// Reverse this relation (swap from and to)
    pub fn rev(self) -> Self {
        Self {
            rel_type: self.rel_type,
            from_tbl: self.to_tbl,
            to_tbl: self.from_tbl,
            from_col: self.to_col,
            to_col: self.from_col,
        }
    }
}

impl<E, R> RelationBuilder<E, R>
where
    E: EntityTrait,
    R: EntityTrait,
{
    pub(crate) fn new(rel_type: RelationType, from: E, to: R) -> Self {
        Self {
            entities: PhantomData,
            rel_type,
            from_tbl: from.into_iden(),
            to_tbl: to.into_iden(),
            from_col: None,
            to_col: None,
        }
    }

    pub fn from(mut self, identifier: E::Column) -> Self {
        self.from_col = Some(identifier.into_identity());
        self
    }

    pub fn to(mut self, identifier: R::Column) -> Self {
        self.to_col = Some(identifier.into_identity());
        self
    }
}

impl<E, R> From<RelationBuilder<E, R>> for RelationDef
where
    E: EntityTrait,
    R: EntityTrait,
{
    fn from(b: RelationBuilder<E, R>) -> Self {
        RelationDef {
            rel_type: b.rel_type,
            from_tbl: b.from_tbl,
            to_tbl: b.to_tbl,
            from_col: b.from_col.unwrap(),
            to_col: b.to_col.unwrap(),
        }
    }
}
