use crate::{
    join_tbl_on_condition, soft_delete_condition_tbl, unpack_table_ref, EntityTrait, QuerySelect,
    RelationDef, Select,
};
use sea_query::{Alias, IntoIden, JoinType, SeaRc};

/// Same as [RelationDef]
pub type LinkDef = RelationDef;

/// A Trait for links between Entities
pub trait Linked {
    #[allow(missing_docs)]
    type FromEntity: EntityTrait;

    #[allow(missing_docs)]
    type ToEntity: EntityTrait;

    /// Link for an Entity
    fn link(&self) -> Vec<LinkDef>;

    /// Find all the Entities that are linked to the Entity
    fn find_linked(&self) -> Select<Self::ToEntity> {
        let mut select = Select::new();
        for (i, rel) in self.link().into_iter().rev().enumerate() {
            let from_tbl = Alias::new(&format!("r{}", i)).into_iden();
            let to_tbl = if i > 0 {
                Alias::new(&format!("r{}", i - 1)).into_iden()
            } else {
                unpack_table_ref(&rel.to_tbl)
            };

            select.query().join_as(
                JoinType::InnerJoin,
                rel.from_tbl,
                SeaRc::clone(&from_tbl),
                soft_delete_condition_tbl(
                    SeaRc::clone(&from_tbl),
                    rel.from_soft_delete_col.as_ref(),
                )
                .add(join_tbl_on_condition(
                    from_tbl,
                    to_tbl,
                    rel.from_col,
                    rel.to_col,
                )),
            );
        }
        select
    }
}
