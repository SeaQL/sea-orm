use crate::{
    ColumnTrait, EntityTrait, Identity, IntoIdentity, IntoSimpleExpr, Iterable, ModelTrait,
    PrimaryKeyToColumn, RelationDef,
};
use sea_query::{
    Alias, Expr, Iden, IntoCondition, LockType, SeaRc, SelectExpr, SelectStatement, SimpleExpr,
    TableRef,
};
pub use sea_query::{Condition, ConditionalStatement, DynIden, JoinType, Order, OrderedStatement};

// LINT: when the column does not appear in tables selected from
// LINT: when there is a group by clause, but some columns don't have aggregate functions
// LINT: when the join table or column does not exists
/// Constraints for any type that needs to perform select statements on a Model
pub trait QuerySelect: Sized {
    #[allow(missing_docs)]
    type QueryStatement;

    /// Add the select SQL statement
    fn query(&mut self) -> &mut SelectStatement;

    /// Clear the selection list
    fn select_only(mut self) -> Self {
        self.query().clear_selects();
        self
    }

    /// Add a select column
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .select_only()
    ///         .column(cake::Column::Name)
    ///         .build(DbBackend::Postgres)
    ///         .to_string(),
    ///     r#"SELECT "cake"."name" FROM "cake""#
    /// );
    /// ```
    fn column<C>(mut self, col: C) -> Self
    where
        C: ColumnTrait,
    {
        self.query().expr(col.into_simple_expr());
        self
    }

    /// Add a select column with alias
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .select_only()
    ///         .column_as(cake::Column::Id.count(), "count")
    ///         .build(DbBackend::Postgres)
    ///         .to_string(),
    ///     r#"SELECT COUNT("cake"."id") AS "count" FROM "cake""#
    /// );
    /// ```
    fn column_as<C, I>(mut self, col: C, alias: I) -> Self
    where
        C: IntoSimpleExpr,
        I: IntoIdentity,
    {
        self.query().expr(SelectExpr {
            expr: col.into_simple_expr(),
            alias: Some(SeaRc::new(alias.into_identity())),
        });
        self
    }

    /// Add an offset expression
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .offset(10)
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` OFFSET 10"
    /// );
    /// ```
    fn offset(mut self, offset: u64) -> Self {
        self.query().offset(offset);
        self
    }

    /// Add a limit expression
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .limit(10)
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` LIMIT 10"
    /// );
    /// ```
    fn limit(mut self, limit: u64) -> Self {
        self.query().limit(limit);
        self
    }

    /// Add a group by column
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .select_only()
    ///         .column(cake::Column::Name)
    ///         .group_by(cake::Column::Name)
    ///         .build(DbBackend::Postgres)
    ///         .to_string(),
    ///     r#"SELECT "cake"."name" FROM "cake" GROUP BY "cake"."name""#
    /// );
    /// ```
    fn group_by<C>(mut self, col: C) -> Self
    where
        C: IntoSimpleExpr,
    {
        self.query().add_group_by(vec![col.into_simple_expr()]);
        self
    }

    /// Add an AND HAVING expression
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .having(cake::Column::Id.eq(4))
    ///         .having(cake::Column::Id.eq(5))
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` HAVING `cake`.`id` = 4 AND `cake`.`id` = 5"
    /// );
    /// ```
    fn having<F>(mut self, filter: F) -> Self
    where
        F: IntoCondition,
    {
        self.query().cond_having(filter.into_condition());
        self
    }

    #[doc(hidden)]
    fn join_join(mut self, join: JoinType, rel: RelationDef, via: Option<RelationDef>) -> Self {
        if let Some(via) = via {
            self = self.join(join, via)
        }
        self.join(join, rel)
    }

    #[doc(hidden)]
    fn join_join_rev(mut self, join: JoinType, rel: RelationDef, via: Option<RelationDef>) -> Self {
        self = self.join_rev(join, rel);
        if let Some(via) = via {
            self = self.join_rev(join, via)
        }
        self
    }

    /// Join via [`RelationDef`].
    fn join(mut self, join: JoinType, rel: RelationDef) -> Self {
        self.query()
            .join(join, rel.to_tbl.clone(), join_condition(rel));
        self
    }

    /// Join via [`RelationDef`] but in reverse direction.
    /// Assume when there exist a relation A to B.
    /// You can reverse join B from A.
    fn join_rev(mut self, join: JoinType, rel: RelationDef) -> Self {
        self.query()
            .join(join, rel.from_tbl.clone(), join_condition(rel));
        self
    }

    /// Select lock
    fn lock(mut self, lock_type: LockType) -> Self {
        self.query().lock(lock_type);
        self
    }

    /// Select lock shared
    fn lock_shared(mut self) -> Self {
        self.query().lock_shared();
        self
    }

    /// Select lock exclusive
    fn lock_exclusive(mut self) -> Self {
        self.query().lock_exclusive();
        self
    }
}

// LINT: when the column does not appear in tables selected from
/// Performs ORDER BY operations
pub trait QueryOrder: Sized {
    #[allow(missing_docs)]
    type QueryStatement: OrderedStatement;

    /// Add the query to perform an ORDER BY operation
    fn query(&mut self) -> &mut SelectStatement;

    /// Add an order_by expression
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .order_by(cake::Column::Id, Order::Asc)
    ///         .order_by(cake::Column::Name, Order::Desc)
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` ORDER BY `cake`.`id` ASC, `cake`.`name` DESC"
    /// );
    /// ```
    fn order_by<C>(mut self, col: C, ord: Order) -> Self
    where
        C: IntoSimpleExpr,
    {
        self.query().order_by_expr(col.into_simple_expr(), ord);
        self
    }

    /// Add an order_by expression (ascending)
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .order_by_asc(cake::Column::Id)
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` ORDER BY `cake`.`id` ASC"
    /// );
    /// ```
    fn order_by_asc<C>(mut self, col: C) -> Self
    where
        C: IntoSimpleExpr,
    {
        self.query()
            .order_by_expr(col.into_simple_expr(), Order::Asc);
        self
    }

    /// Add an order_by expression (descending)
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .order_by_desc(cake::Column::Id)
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` ORDER BY `cake`.`id` DESC"
    /// );
    /// ```
    fn order_by_desc<C>(mut self, col: C) -> Self
    where
        C: IntoSimpleExpr,
    {
        self.query()
            .order_by_expr(col.into_simple_expr(), Order::Desc);
        self
    }
}

// LINT: when the column does not appear in tables selected from
/// Perform a FILTER opertation on a statement
pub trait QueryFilter: Sized {
    #[allow(missing_docs)]
    type QueryStatement: ConditionalStatement;

    /// Add the query to perform a FILTER on
    fn query(&mut self) -> &mut Self::QueryStatement;

    /// Add an AND WHERE expression
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(cake::Column::Id.eq(4))
    ///         .filter(cake::Column::Id.eq(5))
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`id` = 4 AND `cake`.`id` = 5"
    /// );
    /// ```
    ///
    /// Add a condition tree.
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(
    ///             Condition::any()
    ///                 .add(cake::Column::Id.eq(4))
    ///                 .add(cake::Column::Id.eq(5))
    ///         )
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`id` = 4 OR `cake`.`id` = 5"
    /// );
    /// ```
    ///
    /// Add a runtime-built condition tree.
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    /// struct Input {
    ///     name: Option<String>,
    /// }
    /// let input = Input {
    ///     name: Some("cheese".to_owned()),
    /// };
    ///
    /// let mut conditions = Condition::all();
    /// if let Some(name) = input.name {
    ///     conditions = conditions.add(cake::Column::Name.contains(&name));
    /// }
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(conditions)
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`name` LIKE '%cheese%'"
    /// );
    /// ```
    ///
    /// Add a runtime-built condition tree, functional-way.
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    /// struct Input {
    ///     name: Option<String>,
    /// }
    /// let input = Input {
    ///     name: Some("cheese".to_owned()),
    /// };
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(
    ///             Condition::all().add_option(input.name.map(|n| cake::Column::Name.contains(&n)))
    ///         )
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`name` LIKE '%cheese%'"
    /// );
    /// ```
    ///
    /// A slightly more complex example.
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, sea_query::Expr, DbBackend};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(
    ///             Condition::all()
    ///                 .add(
    ///                     Condition::all()
    ///                         .not()
    ///                         .add(Expr::val(1).eq(1))
    ///                         .add(Expr::val(2).eq(2))
    ///                 )
    ///                 .add(
    ///                     Condition::any()
    ///                         .add(Expr::val(3).eq(3))
    ///                         .add(Expr::val(4).eq(4))
    ///                 )
    ///         )
    ///         .build(DbBackend::Postgres)
    ///         .to_string(),
    ///     r#"SELECT "cake"."id", "cake"."name" FROM "cake" WHERE (NOT (1 = 1 AND 2 = 2)) AND (3 = 3 OR 4 = 4)"#
    /// );
    /// ```
    /// Use a sea_query expression
    /// ```
    /// use sea_orm::{entity::*, query::*, sea_query::Expr, tests_cfg::fruit, DbBackend};
    ///
    /// assert_eq!(
    ///     fruit::Entity::find()
    ///         .filter(Expr::col(fruit::Column::CakeId).is_null())
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     "SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` WHERE `cake_id` IS NULL"
    /// );
    /// ```
    fn filter<F>(mut self, filter: F) -> Self
    where
        F: IntoCondition,
    {
        self.query().cond_where(filter.into_condition());
        self
    }

    /// Apply a where condition using the model's primary key
    fn belongs_to<M>(mut self, model: &M) -> Self
    where
        M: ModelTrait,
    {
        for key in <M::Entity as EntityTrait>::PrimaryKey::iter() {
            let col = key.into_column();
            self = self.filter(col.eq(model.get(col)));
        }
        self
    }

    /// Perform a check to determine table belongs to a Model through it's name alias
    fn belongs_to_tbl_alias<M>(mut self, model: &M, tbl_alias: &str) -> Self
    where
        M: ModelTrait,
    {
        for key in <M::Entity as EntityTrait>::PrimaryKey::iter() {
            let col = key.into_column();
            let expr = Expr::tbl(Alias::new(tbl_alias), col).eq(model.get(col));
            self = self.filter(expr);
        }
        self
    }
}

pub(crate) fn join_condition(rel: RelationDef) -> SimpleExpr {
    let from_tbl = unpack_table_ref(&rel.from_tbl);
    let to_tbl = unpack_table_ref(&rel.to_tbl);
    let owner_keys = rel.from_col;
    let foreign_keys = rel.to_col;

    join_tbl_on_condition(from_tbl, to_tbl, owner_keys, foreign_keys)
}

pub(crate) fn join_tbl_on_condition(
    from_tbl: SeaRc<dyn Iden>,
    to_tbl: SeaRc<dyn Iden>,
    owner_keys: Identity,
    foreign_keys: Identity,
) -> SimpleExpr {
    match (owner_keys, foreign_keys) {
        (Identity::Unary(o1), Identity::Unary(f1)) => {
            Expr::tbl(SeaRc::clone(&from_tbl), o1).equals(SeaRc::clone(&to_tbl), f1)
        }
        (Identity::Binary(o1, o2), Identity::Binary(f1, f2)) => {
            Expr::tbl(SeaRc::clone(&from_tbl), o1)
                .equals(SeaRc::clone(&to_tbl), f1)
                .and(Expr::tbl(SeaRc::clone(&from_tbl), o2).equals(SeaRc::clone(&to_tbl), f2))
        }
        (Identity::Ternary(o1, o2, o3), Identity::Ternary(f1, f2, f3)) => {
            Expr::tbl(SeaRc::clone(&from_tbl), o1)
                .equals(SeaRc::clone(&to_tbl), f1)
                .and(Expr::tbl(SeaRc::clone(&from_tbl), o2).equals(SeaRc::clone(&to_tbl), f2))
                .and(Expr::tbl(SeaRc::clone(&from_tbl), o3).equals(SeaRc::clone(&to_tbl), f3))
        }
        _ => panic!("Owner key and foreign key mismatch"),
    }
}

pub(crate) fn unpack_table_ref(table_ref: &TableRef) -> DynIden {
    match table_ref {
        TableRef::Table(tbl) => SeaRc::clone(tbl),
        TableRef::SchemaTable(_, tbl) => SeaRc::clone(tbl),
        TableRef::TableAlias(tbl, _) => SeaRc::clone(tbl),
        TableRef::SchemaTableAlias(_, tbl, _) => SeaRc::clone(tbl),
        TableRef::SubQuery(_, tbl) => SeaRc::clone(tbl),
    }
}
