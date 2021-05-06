use super::{Identity, IntoIdentity};
use crate::Entity;
use sea_query::{Iden, IntoIden};
use std::rc::Rc;

#[derive(Debug)]
pub enum RelationType {
    HasOne,
    HasMany,
    BelongsTo,
}

pub trait Relation {
    fn rel_def() -> RelationDef;
}

pub struct RelationDef {
    rel_type: RelationType,
    from_tbl: Rc<dyn Iden>,
    from_col: Identity,
    to_col: Identity,
}

pub struct RelationBuilder {
    rel_type: RelationType,
    from_tbl: Rc<dyn Iden>,
    from_col: Option<Identity>,
    to_col: Option<Identity>,
}

impl RelationBuilder {
    pub fn has_one<E: 'static>(entity: E) -> Self
    where
        E: Entity,
    {
        Self::new(RelationType::HasOne, entity)
    }

    pub fn has_many<E: 'static>(entity: E) -> Self
    where
        E: Entity,
    {
        Self::new(RelationType::HasMany, entity)
    }

    pub fn belongs_to<E: 'static>(entity: E) -> Self
    where
        E: Entity,
    {
        Self::new(RelationType::BelongsTo, entity)
    }

    fn new<E: 'static>(rel_type: RelationType, entity: E) -> Self
    where
        E: Entity,
    {
        Self {
            rel_type,
            from_tbl: entity.into_iden(),
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
            from_col: b.from_col.unwrap(),
            to_col: b.to_col.unwrap(),
        }
    }
}
