use crate::{EntityTrait, Identity, Iterable, RelationDef, RelationTrait, Statement};
use core::fmt::Debug;
use core::marker::PhantomData;
pub use sea_query::JoinType;
use sea_query::{Expr, Iden, IntoIden, Order, QueryBuilder, SelectStatement, SimpleExpr};
use std::rc::Rc;

#[derive(Debug)]
pub struct Select<E: 'static>
where
    E: EntityTrait,
{
    select: SelectStatement,
    entity: PhantomData<E>,
}

impl<E: 'static> Select<E>
where
    E: EntityTrait,
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

    /// ```
    /// use sea_orm::{ColumnTrait, EntityTrait, tests_cfg::cake, sea_query::MysqlQueryBuilder};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(cake::Column::Id.eq(5))
    ///         .build(MysqlQueryBuilder)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`id` = 5"
    /// );
    /// ```
    pub fn filter(mut self, expr: SimpleExpr) -> Self {
        self.select.and_where(expr);
        self
    }

    /// ```
    /// use sea_orm::{EntityTrait, tests_cfg::cake, sea_query::MysqlQueryBuilder};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .order_by(cake::Column::Id)
    ///         .build(MysqlQueryBuilder)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` ORDER BY `cake`.`id` ASC"
    /// );
    /// ```
    pub fn order_by(mut self, col: E::Column) -> Self {
        self.select.order_by((E::default(), col), Order::Asc);
        self
    }

    /// ```
    /// use sea_orm::{EntityTrait, tests_cfg::cake, sea_query::MysqlQueryBuilder};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .order_by_desc(cake::Column::Id)
    ///         .build(MysqlQueryBuilder)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` ORDER BY `cake`.`id` DESC"
    /// );
    /// ```
    pub fn order_by_desc(mut self, col: E::Column) -> Self {
        self.select.order_by((E::default(), col), Order::Desc);
        self
    }

    pub fn left_join(self, rel: E::Relation) -> Self {
        self.prepare_join(JoinType::LeftJoin, E::Relation::rel_def(&rel))
    }

    pub fn right_join(self, rel: E::Relation) -> Self {
        self.prepare_join(JoinType::RightJoin, E::Relation::rel_def(&rel))
    }

    pub fn inner_join(self, rel: E::Relation) -> Self {
        self.prepare_join(JoinType::InnerJoin, E::Relation::rel_def(&rel))
    }

    /// Get a mutable ref to the query builder
    pub fn query(&mut self) -> &mut SelectStatement {
        &mut self.select
    }

    /// Get a immutable ref to the query builder
    pub fn as_query(&self) -> &SelectStatement {
        &self.select
    }

    /// Take ownership of the query builder
    pub fn into_query(self) -> SelectStatement {
        self.select
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
    use crate::tests_cfg::{cake, fruit};
    use crate::{ColumnTrait, EntityTrait};
    use sea_query::MysqlQueryBuilder;

    #[test]
    fn join_1() {
        assert_eq!(
            cake::Entity::find()
                .left_join(cake::Relation::Fruit)
                .build(MysqlQueryBuilder)
                .to_string(),
            "SELECT `cake`.`id`, `cake`.`name` FROM `cake` LEFT JOIN `fruit` ON `cake`.`id` = `fruit`.`cake_id`"
        );
    }

    #[test]
    fn join_2() {
        assert_eq!(
            cake::Entity::find()
                .inner_join(cake::Relation::Fruit)
                .filter(fruit::Column::Name.contains("cherry"))
                .build(MysqlQueryBuilder)
                .to_string(),
            [
                "SELECT `cake`.`id`, `cake`.`name` FROM `cake`",
                "INNER JOIN `fruit` ON `cake`.`id` = `fruit`.`cake_id`",
                "WHERE `fruit`.`name` LIKE \'%cherry%\'"
            ]
            .join(" ")
        );
    }
}
