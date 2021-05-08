use super::{Identity, IntoIdentity};
use crate::EntityTrait;
use sea_query::{Iden, IntoIden};
use std::rc::Rc;

#[derive(Debug)]
pub enum RelationType {
    HasOne,
    HasMany,
    BelongsTo,
}

pub trait RelationTrait {
    fn rel_def(&self) -> RelationDef;
}

pub struct RelationDef {
    pub rel_type: RelationType,
    pub to_tbl: Rc<dyn Iden>,
    pub from_col: Identity,
    pub to_col: Identity,
}

pub struct RelationBuilder {
    rel_type: RelationType,
    to_tbl: Rc<dyn Iden>,
    from_col: Option<Identity>,
    to_col: Option<Identity>,
}

impl RelationBuilder {
    pub fn has_one<E: 'static>(entity: E) -> Self
    where
        E: EntityTrait,
    {
        Self::new(RelationType::HasOne, entity)
    }

    pub fn has_many<E: 'static>(entity: E) -> Self
    where
        E: EntityTrait,
    {
        Self::new(RelationType::HasMany, entity)
    }

    pub fn belongs_to<E: 'static>(entity: E) -> Self
    where
        E: EntityTrait,
    {
        Self::new(RelationType::BelongsTo, entity)
    }

    fn new<E: 'static>(rel_type: RelationType, entity: E) -> Self
    where
        E: EntityTrait,
    {
        Self {
            rel_type,
            to_tbl: entity.into_iden(),
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
            to_tbl: b.to_tbl,
            from_col: b.from_col.unwrap(),
            to_col: b.to_col.unwrap(),
        }
    }
}
