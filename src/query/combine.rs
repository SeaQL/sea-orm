use crate::{
    ColumnTrait, EntityTrait, IdenStatic, IntoSimpleExpr, Iterable, QueryTrait, Select, SelectTwo,
    SelectTwoMany,
};
use core::marker::PhantomData;
pub use sea_query::JoinType;
use sea_query::{
    Alias, ColumnRef, DynIden, Expr, Iden, Order, SeaRc, SelectExpr, SelectStatement, SimpleExpr,
};

macro_rules! select_def {
    ( $ident: ident, $str: expr ) => {
        /// Implements the traits [Iden] and [IdenStatic] for a type
        #[derive(Debug, Clone, Copy)]
        pub struct $ident;

        impl Iden for $ident {
            fn unquoted(&self, s: &mut dyn std::fmt::Write) {
                write!(s, "{}", self.as_str()).unwrap();
            }
        }

        impl IdenStatic for $ident {
            fn as_str(&self) -> &str {
                $str
            }
        }
    };
}

select_def!(SelectA, "A_");
select_def!(SelectB, "B_");

impl<E> Select<E>
where
    E: EntityTrait,
{
    pub(crate) fn apply_alias(mut self, pre: &str) -> Self {
        self.query().exprs_mut_for_each(|sel| {
            match &sel.alias {
                Some(alias) => {
                    let alias = format!("{}{}", pre, alias.to_string().as_str());
                    sel.alias = Some(SeaRc::new(Alias::new(&alias)));
                }
                None => {
                    let col = match &sel.expr {
                        SimpleExpr::Column(col_ref) => match &col_ref {
                            ColumnRef::Column(col) | ColumnRef::TableColumn(_, col) => col,
                            _ => panic!("Unimplemented"),
                        },
                        SimpleExpr::AsEnum(_, simple_expr) => match simple_expr.as_ref() {
                            SimpleExpr::Column(col_ref) => match &col_ref {
                                ColumnRef::Column(col) | ColumnRef::TableColumn(_, col) => col,
                                _ => panic!(
                                    "cannot apply alias for AsEnum with expr other than Column"
                                ),
                            },
                            _ => {
                                panic!("cannot apply alias for AsEnum with expr other than Column")
                            }
                        },
                        _ => panic!("cannot apply alias for expr other than Column or AsEnum"),
                    };
                    let alias = format!("{}{}", pre, col.to_string().as_str());
                    sel.alias = Some(SeaRc::new(Alias::new(&alias)));
                }
            };
        });
        self
    }

    /// Selects and Entity and returns it together with the Entity from `Self`
    pub fn select_also<F>(mut self, _: F) -> SelectTwo<E, F>
    where
        F: EntityTrait,
    {
        self = self.apply_alias(SelectA.as_str());
        SelectTwo::new(self.into_query())
    }

    /// Makes a SELECT operation in conjunction to another relation
    pub fn select_with<F>(mut self, _: F) -> SelectTwoMany<E, F>
    where
        F: EntityTrait,
    {
        self = self.apply_alias(SelectA.as_str());
        SelectTwoMany::new(self.into_query())
    }
}

impl<E, F> SelectTwo<E, F>
where
    E: EntityTrait,
    F: EntityTrait,
{
    pub(crate) fn new(query: SelectStatement) -> Self {
        Self::new_without_prepare(query).prepare_select()
    }

    pub(crate) fn new_without_prepare(query: SelectStatement) -> Self {
        Self {
            query,
            entity: PhantomData,
        }
    }

    fn prepare_select(mut self) -> Self {
        prepare_select_two::<F, Self>(&mut self);
        self
    }
}

impl<E, F> SelectTwoMany<E, F>
where
    E: EntityTrait,
    F: EntityTrait,
{
    pub(crate) fn new(query: SelectStatement) -> Self {
        Self {
            query,
            entity: PhantomData,
        }
        .prepare_select()
        .prepare_order_by()
    }

    fn prepare_select(mut self) -> Self {
        prepare_select_two::<F, Self>(&mut self);
        self
    }

    fn prepare_order_by(mut self) -> Self {
        for col in <E::PrimaryKey as Iterable>::iter() {
            self.query.order_by((E::default(), col), Order::Asc);
        }
        self
    }
}

fn prepare_select_two<F, S>(selector: &mut S)
where
    F: EntityTrait,
    S: QueryTrait<QueryStatement = SelectStatement>,
{
    let text_type = SeaRc::new(Alias::new("text")) as DynIden;
    for col in <F::Column as Iterable>::iter() {
        let col_def = col.def();
        let col_type = col_def.get_column_type();
        let alias = format!("{}{}", SelectB.as_str(), col.as_str());
        let expr = Expr::expr(col.into_simple_expr());
        let expr = match col_type.get_enum_name() {
            Some(_) => expr.as_enum(text_type.clone()),
            None => expr.into(),
        };
        selector.query().expr(SelectExpr {
            expr,
            alias: Some(SeaRc::new(Alias::new(&alias))),
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_cfg::{cake, fruit};
    use crate::{ColumnTrait, DbBackend, EntityTrait, QueryFilter, QuerySelect, QueryTrait};

    #[test]
    fn alias_1() {
        assert_eq!(
            cake::Entity::find()
                .column_as(cake::Column::Id, "B")
                .apply_alias("A_")
                .build(DbBackend::MySql)
                .to_string(),
            "SELECT `cake`.`id` AS `A_id`, `cake`.`name` AS `A_name`, `cake`.`id` AS `A_B` FROM `cake`",
        );
    }

    #[test]
    fn select_also_1() {
        assert_eq!(
            cake::Entity::find()
                .left_join(fruit::Entity)
                .select_also(fruit::Entity)
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `cake`.`id` AS `A_id`, `cake`.`name` AS `A_name`,",
                "`fruit`.`id` AS `B_id`, `fruit`.`name` AS `B_name`, `fruit`.`cake_id` AS `B_cake_id`",
                "FROM `cake` LEFT JOIN `fruit` ON `cake`.`id` = `fruit`.`cake_id`",
            ].join(" ")
        );
    }

    #[test]
    fn select_with_1() {
        assert_eq!(
            cake::Entity::find()
                .left_join(fruit::Entity)
                .select_with(fruit::Entity)
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `cake`.`id` AS `A_id`, `cake`.`name` AS `A_name`,",
                "`fruit`.`id` AS `B_id`, `fruit`.`name` AS `B_name`, `fruit`.`cake_id` AS `B_cake_id`",
                "FROM `cake` LEFT JOIN `fruit` ON `cake`.`id` = `fruit`.`cake_id`",
                "ORDER BY `cake`.`id` ASC",
            ].join(" ")
        );
    }

    #[test]
    fn select_also_2() {
        assert_eq!(
            cake::Entity::find()
                .left_join(fruit::Entity)
                .select_also(fruit::Entity)
                .filter(cake::Column::Id.eq(1))
                .filter(fruit::Column::Id.eq(2))
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `cake`.`id` AS `A_id`, `cake`.`name` AS `A_name`,",
                "`fruit`.`id` AS `B_id`, `fruit`.`name` AS `B_name`, `fruit`.`cake_id` AS `B_cake_id`",
                "FROM `cake` LEFT JOIN `fruit` ON `cake`.`id` = `fruit`.`cake_id`",
                "WHERE `cake`.`id` = 1 AND `fruit`.`id` = 2",
            ].join(" ")
        );
    }

    #[test]
    fn select_with_2() {
        assert_eq!(
            cake::Entity::find()
                .left_join(fruit::Entity)
                .select_with(fruit::Entity)
                .filter(cake::Column::Id.eq(1))
                .filter(fruit::Column::Id.eq(2))
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `cake`.`id` AS `A_id`, `cake`.`name` AS `A_name`,",
                "`fruit`.`id` AS `B_id`, `fruit`.`name` AS `B_name`, `fruit`.`cake_id` AS `B_cake_id`",
                "FROM `cake` LEFT JOIN `fruit` ON `cake`.`id` = `fruit`.`cake_id`",
                "WHERE `cake`.`id` = 1 AND `fruit`.`id` = 2",
                "ORDER BY `cake`.`id` ASC",
            ].join(" ")
        );
    }
}
