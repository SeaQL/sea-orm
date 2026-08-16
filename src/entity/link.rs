use crate::{EntityTrait, QuerySelect, RelationDef, Select, join_tbl_on_condition};
use sea_query::{
    Alias, CommonTableExpression, Condition, IntoIden, IntoTableRef, JoinType, UnionType,
};

/// One hop in a multi-hop [`Linked`] chain. Alias for [`RelationDef`].
pub type LinkDef = RelationDef;

/// A multi-hop traversal between two entities: a chain of [`LinkDef`] hops
/// from `FromEntity` to `ToEntity`.
///
/// Use this when a single [`Related`](crate::Related) edge can't express
/// the path (for example, "user → post → comment → author"). Implement
/// [`link`](Self::link) to return the ordered list of hops; SeaORM joins
/// them when [`ModelTrait::find_linked`](crate::ModelTrait::find_linked) is
/// called.
pub trait Linked {
    /// Entity at the start of the chain.
    type FromEntity: EntityTrait;

    /// Entity reached at the end of the chain.
    type ToEntity: EntityTrait;

    /// Ordered chain of hops from `FromEntity` to `ToEntity`.
    fn link(&self) -> Vec<LinkDef>;

    /// Build a [`Select<ToEntity>`] that follows the chain.
    fn find_linked(&self) -> Select<Self::ToEntity> {
        find_linked(self.link().into_iter().rev(), JoinType::InnerJoin)
    }
}

pub(crate) fn find_linked<I, E>(links: I, join: JoinType) -> Select<E>
where
    I: Iterator<Item = LinkDef>,
    E: EntityTrait,
{
    let mut select = Select::new();
    for (i, mut rel) in links.enumerate() {
        let from_tbl = format!("r{i}").into_iden();
        let to_tbl = if i > 0 {
            format!("r{}", i - 1).into_iden()
        } else {
            rel.to_tbl.sea_orm_table().clone()
        };
        let table_ref = rel.from_tbl;

        let mut condition = Condition::all().add(join_tbl_on_condition(
            from_tbl.clone(),
            to_tbl.clone(),
            rel.from_col,
            rel.to_col,
        ));
        if let Some(f) = rel.on_condition.take() {
            condition = condition.add(f(from_tbl.clone(), to_tbl.clone()));
        }

        select.query().join_as(join, table_ref, from_tbl, condition);
    }
    select
}

pub(crate) fn find_linked_recursive<E>(
    mut initial_query: Select<E>,
    mut link: Vec<LinkDef>,
) -> Select<E>
where
    E: EntityTrait,
{
    let cte_name = Alias::new("cte");

    let Some(first) = link.first_mut() else {
        return initial_query;
    };
    first.from_tbl = cte_name.clone().into_table_ref();
    let mut recursive_query: Select<E> =
        find_linked(link.into_iter().rev(), JoinType::InnerJoin).select_only();
    initial_query.query.exprs_mut_for_each(|expr| {
        recursive_query.query.expr(expr.clone());
    });

    let mut cte_query = initial_query.query.clone();
    cte_query.union(UnionType::All, recursive_query.query);

    let cte = CommonTableExpression::new()
        .table_name(cte_name.clone())
        .query(cte_query)
        .to_owned();

    let mut select = E::find().select_only();
    initial_query.query.exprs_mut_for_each(|expr| {
        select.query.expr(expr.clone());
    });
    select
        .query
        .from_clear()
        .from_as(cte_name, E::default())
        .with_cte(cte);
    select
}
