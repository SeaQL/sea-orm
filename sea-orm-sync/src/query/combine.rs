use crate::{
    ColumnTrait, EntityTrait, IdenStatic, Iterable, QueryTrait, Select, SelectTwo, SelectTwoMany,
    SelectTwoRequired,
};
use core::marker::PhantomData;
use sea_query::{Iden, IntoIden, Order, SelectExpr, SelectStatement, SimpleExpr};
use std::borrow::Cow;

macro_rules! select_def {
    ( $ident: ident, $str: expr ) => {
        /// Iden used to alias one side of a multi-table `SELECT`, so column
        /// names from each joined table can be prefixed without clashing
        /// (e.g. `select_a_id`, `select_b_id`).
        #[derive(Debug, Clone, Copy)]
        pub struct $ident;

        impl Iden for $ident {
            fn quoted(&self) -> Cow<'static, str> {
                Cow::Borrowed(IdenStatic::as_str(self))
            }

            fn unquoted(&self) -> &str {
                IdenStatic::as_str(self)
            }
        }

        impl IdenStatic for $ident {
            fn as_str(&self) -> &'static str {
                $str
            }
        }
    };
}

select_def!(SelectA, "A_");
select_def!(SelectB, "B_");
select_def!(SelectC, "C_");
select_def!(SelectD, "D_");
select_def!(SelectE, "E_");
select_def!(SelectF, "F_");

impl<E> Select<E>
where
    E: EntityTrait,
{
    pub(crate) fn apply_alias(mut self, pre: &str) -> Self {
        self.query().exprs_mut_for_each(|sel| {
            match &sel.alias {
                Some(alias) => {
                    let alias = format!("{}{}", pre, alias.to_string().as_str());
                    sel.alias = Some(alias.into_iden());
                }
                None => {
                    let col = match &sel.expr {
                        SimpleExpr::Column(col_ref) => match col_ref.column() {
                            Some(col) => col,
                            None => {
                                panic!("cannot apply alias for Column with asterisk");
                            }
                        },
                        SimpleExpr::AsEnum(_, simple_expr) => match simple_expr.as_ref() {
                            SimpleExpr::Column(col_ref) => match col_ref.column() {
                                Some(col) => col,
                                None => {
                                    panic!("cannot apply alias for AsEnum with asterisk")
                                }
                            },
                            _ => {
                                panic!("cannot apply alias for AsEnum with expr other than Column")
                            }
                        },
                        _ => panic!("cannot apply alias for expr other than Column or AsEnum"),
                    };
                    let alias = format!("{}{}", pre, col.to_string().as_str());
                    sel.alias = Some(alias.into_iden());
                }
            };
        });
        self
    }

    /// Selects extra Entity and returns it together with the Entity from `Self`
    pub fn select_also<F>(mut self, _: F) -> SelectTwo<E, F>
    where
        F: EntityTrait,
    {
        self = self.apply_alias(SelectA.as_str());
        SelectTwo::new(self.into_query())
    }

    /// Only used by Entity loader
    #[doc(hidden)]
    pub fn select_also_fake<F>(mut self, _: F) -> SelectTwo<E, F>
    where
        F: EntityTrait,
    {
        self = self.apply_alias(SelectA.as_str());
        SelectTwo::new_without_prepare(self.into_query())
    }

    /// Makes a SELECT operation in conjunction to another relation
    pub fn select_with<F>(mut self, _: F) -> SelectTwoMany<E, F>
    where
        F: EntityTrait,
    {
        self = self.apply_alias(SelectA.as_str());
        SelectTwoMany::new(self.into_query())
    }

    /// Selects extra Entity and returns it together with the Entity from `Self`
    pub fn select_two_required<F>(mut self, _: F) -> SelectTwoRequired<E, F>
    where
        F: EntityTrait,
    {
        self = self.apply_alias(SelectA.as_str());
        SelectTwoRequired::new(self.into_query())
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
        prepare_select_col::<F, _, _>(&mut self, SelectB);
        self
    }
}

impl<E, F> SelectTwoMany<E, F>
where
    E: EntityTrait,
    F: EntityTrait,
{
    pub(crate) fn new(query: SelectStatement) -> Self {
        Self::new_without_prepare(query)
            .prepare_select()
            .prepare_order_by()
    }

    pub(crate) fn new_without_prepare(query: SelectStatement) -> Self {
        Self {
            query,
            entity: PhantomData,
        }
    }

    fn prepare_select(mut self) -> Self {
        prepare_select_col::<F, _, _>(&mut self, SelectB);
        self
    }

    fn prepare_order_by(mut self) -> Self {
        for col in <E::PrimaryKey as Iterable>::iter() {
            self.query.order_by((E::default(), col), Order::Asc);
        }
        self
    }
}

impl<E, F> SelectTwoRequired<E, F>
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
        prepare_select_col::<F, Self, _>(&mut self, SelectB);
        self
    }
}

pub(crate) fn prepare_select_col<F, S, A>(selector: &mut S, alias: A)
where
    F: EntityTrait,
    S: QueryTrait<QueryStatement = SelectStatement>,
    A: IdenStatic,
{
    for col in <F::Column as Iterable>::iter() {
        let alias = format!("{}{}", alias.as_str(), col.as_str());
        selector.query().expr(SelectExpr {
            expr: col.select_as(col.into_expr()),
            alias: Some(alias.into_iden()),
            window: None,
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_cfg::{cake, fruit};
    use crate::{ColumnTrait, DbBackend, EntityTrait, QueryFilter, QuerySelect, QueryTrait};

    #[cfg(feature = "macros")]
    mod select_as_entity {
        use crate as sea_orm;
        use crate::entity::prelude::*;
        use crate::tests_cfg::cake;

        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "select_as_entity")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            pub cake_id: i32,
            #[sea_orm(select_as = "text")]
            pub value: String,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {
            #[sea_orm(
                belongs_to = "cake::Entity",
                from = "Column::CakeId",
                to = "cake::Column::Id"
            )]
            Cake,
        }

        impl Related<cake::Entity> for Entity {
            fn to() -> RelationDef {
                Relation::Cake.def()
            }
        }

        impl ActiveModelBehavior for ActiveModel {}
    }

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
    #[cfg(feature = "macros")]
    fn find_also_related_with_select_as_1() {
        assert_eq!(
            select_as_entity::Entity::find()
                .find_also_related(cake::Entity)
                .build(DbBackend::Postgres)
                .to_string(),
            [
                r#"SELECT "select_as_entity"."id" AS "A_id", "select_as_entity"."cake_id" AS "A_cake_id", CAST("select_as_entity"."value" AS text) AS "A_value","#,
                r#""cake"."id" AS "B_id", "cake"."name" AS "B_name""#,
                r#"FROM "select_as_entity" LEFT JOIN "cake" ON "select_as_entity"."cake_id" = "cake"."id""#,
            ]
            .join(" ")
        );
    }

    #[test]
    #[cfg(feature = "macros")]
    fn find_also_related_with_selected_select_as_column_1() {
        assert_eq!(
            select_as_entity::Entity::find()
                .select_only()
                .column(select_as_entity::Column::Value)
                .find_also_related(cake::Entity)
                .build(DbBackend::Postgres)
                .to_string(),
            [
                r#"SELECT CAST("select_as_entity"."value" AS text) AS "A_value","#,
                r#""cake"."id" AS "B_id", "cake"."name" AS "B_name""#,
                r#"FROM "select_as_entity" LEFT JOIN "cake" ON "select_as_entity"."cake_id" = "cake"."id""#,
            ]
            .join(" ")
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
