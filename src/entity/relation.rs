use crate::{EntityTrait, Identity, IntoIdentity, Select};
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
        Select::<R>::new().join_rev(JoinType::InnerJoin, Self::to())
    }
}

pub struct RelationDef {
    pub rel_type: RelationType,
    pub from_tbl: Rc<dyn Iden>,
    pub to_tbl: Rc<dyn Iden>,
    pub from_col: Identity,
    pub to_col: Identity,
}

pub struct RelationBuilder {
    rel_type: RelationType,
    from_tbl: Rc<dyn Iden>,
    to_tbl: Rc<dyn Iden>,
    from_col: Option<Identity>,
    to_col: Option<Identity>,
}

impl RelationBuilder {
    pub(crate) fn new<E, T>(rel_type: RelationType, from: E, to: T) -> Self
    where
        E: IntoIden,
        T: IntoIden,
    {
        Self {
            rel_type,
            from_tbl: from.into_iden(),
            to_tbl: to.into_iden(),
            from_col: None,
            to_col: None,
        }
    }

    pub fn from<T>(mut self, identifier: T) -> Self
    where
        T: IntoIdentity,
    {
        self.from_col = Some(identifier.into_identity());
        self
    }

    pub fn to<T>(mut self, identifier: T) -> Self
    where
        T: IntoIdentity,
    {
        self.to_col = Some(identifier.into_identity());
        self
    }
}

impl From<RelationBuilder> for RelationDef {
    fn from(b: RelationBuilder) -> Self {
        RelationDef {
            rel_type: b.rel_type,
            from_tbl: b.from_tbl,
            to_tbl: b.to_tbl,
            from_col: b.from_col.unwrap(),
            to_col: b.to_col.unwrap(),
        }
    }
}
