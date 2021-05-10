use crate::{EntityName, IdenStatic};
pub use sea_query::ColumnType;
use sea_query::{Expr, Iden, SimpleExpr, Value};

use std::rc::Rc;

macro_rules! bind_oper {
    ( $op: ident ) => {
        fn $op<V>(&self, v: V) -> SimpleExpr
        where
            V: Into<Value>,
        {
            Expr::tbl(self.entity_name(), *self).$op(v)
        }
    };
}

pub trait ColumnTrait: IdenStatic {
    type EntityName: EntityName;

    fn def(&self) -> ColumnType;

    fn entity_name(&self) -> Rc<dyn Iden> {
        Rc::new(Self::EntityName::default()) as Rc<dyn Iden>
    }

    bind_oper!(eq);
    bind_oper!(ne);
    bind_oper!(gt);
    bind_oper!(gte);
    bind_oper!(lt);
    bind_oper!(lte);

    /// ```
    /// use sea_orm::{ColumnTrait, EntityTrait, tests_cfg::cake, sea_query::MysqlQueryBuilder};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(cake::Column::Id.between(2,3))
    ///         .build(MysqlQueryBuilder)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`id` BETWEEN 2 AND 3"
    /// );
    /// ```
    fn between<V>(&self, a: V, b: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        Expr::tbl(self.entity_name(), *self).between(a, b)
    }

    /// ```
    /// use sea_orm::{ColumnTrait, EntityTrait, tests_cfg::cake, sea_query::MysqlQueryBuilder};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(cake::Column::Id.not_between(2,3))
    ///         .build(MysqlQueryBuilder)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`id` NOT BETWEEN 2 AND 3"
    /// );
    /// ```
    fn not_between<V>(&self, a: V, b: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        Expr::tbl(self.entity_name(), *self).not_between(a, b)
    }

    /// ```
    /// use sea_orm::{ColumnTrait, EntityTrait, tests_cfg::cake, sea_query::MysqlQueryBuilder};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(cake::Column::Name.like("cheese"))
    ///         .build(MysqlQueryBuilder)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`name` LIKE 'cheese'"
    /// );
    /// ```
    fn like(&self, s: &str) -> SimpleExpr {
        Expr::tbl(self.entity_name(), *self).like(s)
    }

    /// ```
    /// use sea_orm::{ColumnTrait, EntityTrait, tests_cfg::cake, sea_query::MysqlQueryBuilder};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(cake::Column::Name.not_like("cheese"))
    ///         .build(MysqlQueryBuilder)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`name` NOT LIKE 'cheese'"
    /// );
    /// ```
    fn not_like(&self, s: &str) -> SimpleExpr {
        Expr::tbl(self.entity_name(), *self).not_like(s)
    }

    /// ```
    /// use sea_orm::{ColumnTrait, EntityTrait, tests_cfg::cake, sea_query::MysqlQueryBuilder};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(cake::Column::Name.starts_with("cheese"))
    ///         .build(MysqlQueryBuilder)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`name` LIKE 'cheese%'"
    /// );
    /// ```
    fn starts_with(&self, s: &str) -> SimpleExpr {
        let pattern = format!("{}%", s);
        Expr::tbl(self.entity_name(), *self).like(&pattern)
    }

    /// ```
    /// use sea_orm::{ColumnTrait, EntityTrait, tests_cfg::cake, sea_query::MysqlQueryBuilder};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(cake::Column::Name.ends_with("cheese"))
    ///         .build(MysqlQueryBuilder)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`name` LIKE '%cheese'"
    /// );
    /// ```
    fn ends_with(&self, s: &str) -> SimpleExpr {
        let pattern = format!("%{}", s);
        Expr::tbl(self.entity_name(), *self).like(&pattern)
    }

    /// ```
    /// use sea_orm::{ColumnTrait, EntityTrait, tests_cfg::cake, sea_query::MysqlQueryBuilder};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(cake::Column::Name.contains("cheese"))
    ///         .build(MysqlQueryBuilder)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`name` LIKE '%cheese%'"
    /// );
    /// ```
    fn contains(&self, s: &str) -> SimpleExpr {
        let pattern = format!("%{}%", s);
        Expr::tbl(self.entity_name(), *self).like(&pattern)
    }
}
