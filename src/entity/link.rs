use crate::{
    join_tbl_on_condition, unpack_table_ref, EntityTrait, Identity, QuerySelect, RelationDef,
    Select,
};
use sea_query::{Alias, Condition, ConditionExpression, IntoIden, JoinType, SeaRc, TableRef};

/// Defines a link between tables
#[derive(Debug)]
pub struct LinkDef {
    /// Reference from another Entity
    pub from_tbl: TableRef,
    /// Reference to another ENtity
    pub to_tbl: TableRef,
    /// Reference to from a Column
    pub from_col: Identity,
    /// Reference to another column
    pub to_col: Identity,
    /// On condition
    pub on_condition: Option<Condition>,
}

/// Convert any types into [`LinkDef`]
pub trait IntoLinkDef {
    /// Perform the conversion
    fn into_link_def(self) -> LinkDef;
}

impl IntoLinkDef for RelationDef {
    fn into_link_def(self) -> LinkDef {
        LinkDef {
            from_tbl: self.from_tbl,
            to_tbl: self.to_tbl,
            from_col: self.from_col,
            to_col: self.to_col,
            on_condition: None,
        }
    }
}

impl From<RelationDef> for LinkDef {
    fn from(rel: RelationDef) -> Self {
        rel.into_link_def()
    }
}

impl LinkDef {
    /// Add an AND on condition in join expression.
    /// Calling `or_on_condition` after `and_on_condition` will conjoin the conditional expression with AND operator.
    pub fn and_on_condition<C>(mut self, cond_expr: C) -> Self
    where
        C: Into<ConditionExpression>,
    {
        self.on_condition = Some(
            match self.on_condition {
                Some(on_condition) => on_condition,
                None => Condition::all(),
            }
            .add(cond_expr),
        );
        self
    }

    /// Add an OR on condition in join expression.
    /// Calling `and_on_condition` after `or_on_condition` will conjoin the conditional expression with OR operator.
    pub fn or_on_condition<C>(mut self, cond_expr: C) -> Self
    where
        C: Into<ConditionExpression>,
    {
        self.on_condition = Some(
            match self.on_condition {
                Some(on_condition) => on_condition,
                None => Condition::any(),
            }
            .add(cond_expr),
        );
        self
    }
}

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

            let condition = match rel.on_condition {
                Some(condition) => condition,
                None => Condition::all(),
            }
            .add(join_tbl_on_condition(
                SeaRc::clone(&from_tbl),
                to_tbl,
                rel.from_col,
                rel.to_col,
            ));

            select
                .query()
                .join_as(JoinType::InnerJoin, rel.from_tbl, from_tbl, condition);
        }
        select
    }
}
