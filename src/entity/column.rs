use crate::{EntityName, IdenStatic, Iterable};
pub use sea_query::ColumnType;
use sea_query::{DynIden, Expr, SeaRc, SimpleExpr, Value};

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

macro_rules! bind_agg_func {
    ( $func: ident ) => {
        fn $func(&self) -> SimpleExpr {
            Expr::tbl(self.entity_name(), *self).$func()
        }
    };
}

macro_rules! bind_vec_func {
    ( $func: ident ) => {
        #[allow(clippy::wrong_self_convention)]
        fn $func<V, I>(&self, v: I) -> SimpleExpr
        where
            V: Into<Value>,
            I: IntoIterator<Item = V>,
        {
            Expr::tbl(self.entity_name(), *self).$func(v)
        }
    };
}

// LINT: when the operand value does not match column type
pub trait ColumnTrait: IdenStatic + Iterable {
    type EntityName: EntityName;

    fn def(&self) -> ColumnType;

    // we may add more:
    // fn indexed(&self) -> bool;
    // fn unique(&self) -> bool;

    fn entity_name(&self) -> DynIden {
        SeaRc::new(Self::EntityName::default()) as DynIden
    }

    fn as_column_ref(&self) -> (DynIden, DynIden) {
        (self.entity_name(), SeaRc::new(*self) as DynIden)
    }

    bind_oper!(eq);
    bind_oper!(ne);
    bind_oper!(gt);
    bind_oper!(gte);
    bind_oper!(lt);
    bind_oper!(lte);

    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, sea_query::MysqlQueryBuilder};
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
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, sea_query::MysqlQueryBuilder};
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
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, sea_query::MysqlQueryBuilder};
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
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, sea_query::MysqlQueryBuilder};
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
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, sea_query::MysqlQueryBuilder};
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
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, sea_query::MysqlQueryBuilder};
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
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, sea_query::MysqlQueryBuilder};
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

    bind_agg_func!(max);
    bind_agg_func!(min);
    bind_agg_func!(sum);
    bind_agg_func!(count);

    fn if_null<V>(&self, v: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        Expr::tbl(self.entity_name(), *self).if_null(v)
    }

    bind_vec_func!(is_in);
    bind_vec_func!(is_not_in);
}
