use crate::{ColumnTrait, EntityTrait, Iterable, QueryHelper, Statement};
use core::fmt::Debug;
use core::marker::PhantomData;
pub use sea_query::JoinType;
use sea_query::{
    Alias, ColumnRef, Iden, IntoColumnRef, IntoIden, QueryBuilder, SelectStatement, SimpleExpr,
};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Select<E>
where
    E: EntityTrait,
{
    pub(crate) query: SelectStatement,
    pub(crate) entity: PhantomData<E>,
}

#[derive(Clone, Debug)]
pub struct SelectTwo<E, F>
where
    E: EntityTrait,
    F: EntityTrait,
{
    pub(crate) query: SelectStatement,
    pub(crate) entity: PhantomData<(E, F)>,
}

pub trait IntoSimpleExpr {
    fn into_simple_expr(self) -> SimpleExpr;
}

impl<E> QueryHelper for Select<E>
where
    E: EntityTrait,
{
    fn query(&mut self) -> &mut SelectStatement {
        &mut self.query
    }
}

impl<E, F> QueryHelper for SelectTwo<E, F>
where
    E: EntityTrait,
    F: EntityTrait,
{
    fn query(&mut self) -> &mut SelectStatement {
        &mut self.query
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

impl<E> Select<E>
where
    E: EntityTrait,
{
    pub(crate) fn new() -> Self {
        Self {
            query: SelectStatement::new(),
            entity: PhantomData,
        }
        .prepare_select()
        .prepare_from()
    }

    fn prepare_select(mut self) -> Self {
        self.query.columns(self.column_list());
        self
    }

    fn column_list(&self) -> Vec<(Rc<dyn Iden>, E::Column)> {
        let table = Rc::new(E::default()) as Rc<dyn Iden>;
        E::Column::iter().map(|col| (table.clone(), col)).collect()
    }

    fn prepare_from(mut self) -> Self {
        self.query.from(E::default().into_iden());
        self
    }

    pub(crate) fn apply_alias(mut self, pre: &str) -> Self {
        self.query().exprs_mut_for_each(|sel| {
            match &sel.alias {
                Some(alias) => {
                    let alias = format!("{}{}", pre, alias.to_string().as_str());
                    sel.alias = Some(Rc::new(Alias::new(&alias)));
                }
                None => {
                    let col = match &sel.expr {
                        SimpleExpr::Column(col_ref) => match &col_ref {
                            ColumnRef::Column(col) => col,
                            ColumnRef::TableColumn(_, col) => col,
                        },
                        _ => panic!("cannot apply alias for expr other than Column"),
                    };
                    let alias = format!("{}{}", pre, col.to_string().as_str());
                    sel.alias = Some(Rc::new(Alias::new(&alias)));
                }
            };
        });
        self
    }

    /// Get a mutable ref to the query builder
    pub fn query(&mut self) -> &mut SelectStatement {
        &mut self.query
    }

    /// Get an immutable ref to the query builder
    pub fn as_query(&self) -> &SelectStatement {
        &self.query
    }

    /// Take ownership of the query builder
    pub fn into_query(self) -> SelectStatement {
        self.query
    }

    /// Build the query as [`Statement`]
    pub fn build<B>(&self, builder: B) -> Statement
    where
        B: QueryBuilder,
    {
        self.as_query().build(builder).into()
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_cfg::cake;
    use crate::{EntityTrait, QueryHelper};
    use sea_query::MysqlQueryBuilder;

    #[test]
    fn alias_1() {
        assert_eq!(
            cake::Entity::find()
                .column_as(cake::Column::Id, "B")
                .apply_alias("A_")
                .build(MysqlQueryBuilder)
                .to_string(),
            "SELECT `cake`.`id` AS `A_id`, `cake`.`name` AS `A_name`, `cake`.`id` AS `A_B` FROM `cake`",
        );
    }
}
