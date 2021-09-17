use crate::{EntityTrait, QuerySelect, RelationDef, Select};
use sea_query::JoinType;

pub type LinkDef = RelationDef;

pub trait Linked {
    type FromEntity: EntityTrait;

    type ToEntity: EntityTrait;

    fn link(&self) -> Vec<LinkDef>;

    fn find_linked(&self) -> Select<Self::ToEntity> {
        let mut select = Select::new();
        for rel in self.link().into_iter().rev() {
            select = select.join_rev(JoinType::InnerJoin, rel);
        }
        select
    }
}
