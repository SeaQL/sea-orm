use crate::{EntityTrait, Identity, Iterable, RelationDef, Statement};
use core::fmt::Debug;
use core::marker::PhantomData;
pub use sea_query::JoinType;
use sea_query::{Expr, Iden, IntoIden, Order, QueryBuilder, SelectStatement, SimpleExpr};
use std::rc::Rc;

#[derive(Debug)]
pub struct Select<'s, E: 'static>
where
    E: EntityTrait,
{
    select: SelectStatement,
    entity: PhantomData<&'s E>,
}

impl<E: 'static> Select<'_, E>
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

    pub fn filter(mut self, expr: SimpleExpr) -> Self {
        self.select.and_where(expr);
        self
    }

    pub fn order_by(mut self, col: E::Column) -> Self {
        self.select.order_by((E::default(), col), Order::Asc);
        self
    }

    pub fn order_by_desc(mut self, col: E::Column) -> Self {
        self.select.order_by((E::default(), col), Order::Desc);
        self
    }

    pub fn left_join(self, relation: RelationDef) -> Self {
        self.prepare_join(JoinType::LeftJoin, relation)
    }

    pub fn right_join(self, relation: RelationDef) -> Self {
        self.prepare_join(JoinType::RightJoin, relation)
    }

    pub fn inner_join(self, relation: RelationDef) -> Self {
        self.prepare_join(JoinType::InnerJoin, relation)
    }

    pub fn query(&mut self) -> &mut SelectStatement {
        &mut self.select
    }

    pub fn as_query(&self) -> &SelectStatement {
        &self.select
    }

    pub fn into_query(self) -> SelectStatement {
        self.select
    }

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
    use crate::{ColumnTrait, EntityTrait};
    use sea_query::MysqlQueryBuilder;

    #[test]
    fn test_1() {
        assert_eq!(
            cake::Entity::find().build(MysqlQueryBuilder).to_string(),
            "SELECT `cake`.`id`, `cake`.`name` FROM `cake`"
        );
    }

    #[test]
    fn test_2() {
        assert_eq!(
            cake::Entity::find()
                .filter(cake::Column::Id.eq(5))
                .build(MysqlQueryBuilder)
                .to_string(),
            "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`id` = 5"
        );
    }

    #[test]
    fn test_3() {
        assert_eq!(
            cake::Entity::find()
                .filter(cake::Column::Name.like("cheese"))
                .build(MysqlQueryBuilder)
                .to_string(),
            "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`name` LIKE 'cheese'"
        );
        assert_eq!(
            cake::Entity::find()
                .filter(cake::Column::Name.starts_with("cheese"))
                .build(MysqlQueryBuilder)
                .to_string(),
            "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`name` LIKE 'cheese%'"
        );
        assert_eq!(
            cake::Entity::find()
                .filter(cake::Column::Name.ends_with("cheese"))
                .build(MysqlQueryBuilder)
                .to_string(),
            "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`name` LIKE '%cheese'"
        );
        assert_eq!(
            cake::Entity::find()
                .filter(cake::Column::Name.contains("cheese"))
                .build(MysqlQueryBuilder)
                .to_string(),
            "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`name` LIKE '%cheese%'"
        );
    }

    #[test]
    fn test_4() {
        assert_eq!(
            cake::Entity::find_one()
                .build(MysqlQueryBuilder)
                .to_string(),
            "SELECT `cake`.`id`, `cake`.`name` FROM `cake` LIMIT 1"
        );
    }

    #[test]
    fn test_5() {
        assert_eq!(
            cake::Entity::find_one_by(11)
                .build(MysqlQueryBuilder)
                .to_string(),
            "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`id` = 11 LIMIT 1"
        );
    }

    #[test]
    fn test_6() {
        assert_eq!(
            cake::Entity::find()
                .order_by(cake::Column::Id)
                .build(MysqlQueryBuilder)
                .to_string(),
            "SELECT `cake`.`id`, `cake`.`name` FROM `cake` ORDER BY `cake`.`id` ASC"
        );
        assert_eq!(
            cake::Entity::find()
                .order_by_desc(cake::Column::Id)
                .build(MysqlQueryBuilder)
                .to_string(),
            "SELECT `cake`.`id`, `cake`.`name` FROM `cake` ORDER BY `cake`.`id` DESC"
        );
    }
}
