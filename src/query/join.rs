use crate::{
    join_tbl_on_condition, unpack_table_ref, EntityTrait, IdenStatic, Iterable, Linked, ModelTrait,
    QuerySelect, Related, Select, SelectA, SelectB, SelectTwo, SelectTwoMany,
};
pub use sea_query::JoinType;
use sea_query::{Alias, Expr, IntoIden, SeaRc, SelectExpr};

macro_rules! filter_soft_deleted {
    ( $select: ident, $r: ty ) => {
        match <<$r as EntityTrait>::Model as ModelTrait>::soft_delete_column() {
            Some(soft_delete_column) if !$select.with_deleted => {
                $select
                    .query()
                    .and_where(Expr::tbl(<$r>::default(), soft_delete_column).is_null());
            }
            _ => {}
        }
    };
}

impl<E> Select<E>
where
    E: EntityTrait,
{
    /// Left Join with a Related Entity.
    pub fn left_join<R>(self, _: R) -> Self
    where
        R: EntityTrait,
        E: Related<R>,
    {
        let mut select = self.join_join(JoinType::LeftJoin, E::to(), E::via());
        filter_soft_deleted!(select, R);
        select
    }

    /// Right Join with a Related Entity.
    pub fn right_join<R>(self, _: R) -> Self
    where
        R: EntityTrait,
        E: Related<R>,
    {
        let mut select = self.join_join(JoinType::RightJoin, E::to(), E::via());
        filter_soft_deleted!(select, R);
        select
    }

    /// Inner Join with a Related Entity.
    pub fn inner_join<R>(self, _: R) -> Self
    where
        R: EntityTrait,
        E: Related<R>,
    {
        let mut select = self.join_join(JoinType::InnerJoin, E::to(), E::via());
        filter_soft_deleted!(select, R);
        select
    }

    /// Join with an Entity Related to me.
    pub fn reverse_join<R>(self, _: R) -> Self
    where
        R: EntityTrait + Related<E>,
    {
        let mut select = self.join_rev(JoinType::InnerJoin, R::to());
        filter_soft_deleted!(select, R);
        select
    }

    /// Left Join with a Related Entity and select both Entity.
    pub fn find_also_related<R>(self, r: R) -> SelectTwo<E, R>
    where
        R: EntityTrait,
        E: Related<R>,
    {
        self.left_join(r).select_also(r)
    }

    /// Left Join with a Related Entity and select the related Entity as a `Vec`
    pub fn find_with_related<R>(self, r: R) -> SelectTwoMany<E, R>
    where
        R: EntityTrait,
        E: Related<R>,
    {
        self.left_join(r).select_with(r)
    }

    /// Left Join with a Linked Entity and select both Entity.
    pub fn find_also_linked<L, T>(self, l: L) -> SelectTwo<E, T>
    where
        L: Linked<FromEntity = E, ToEntity = T>,
        T: EntityTrait,
    {
        let with_deleted = self.with_deleted;
        let mut slf = self;
        for (i, rel) in l.link().into_iter().enumerate() {
            let to_tbl = Alias::new(&format!("r{}", i)).into_iden();
            let from_tbl = if i > 0 {
                Alias::new(&format!("r{}", i - 1)).into_iden()
            } else {
                unpack_table_ref(&rel.from_tbl)
            };

            slf.query().join_as(
                JoinType::LeftJoin,
                unpack_table_ref(&rel.to_tbl),
                SeaRc::clone(&to_tbl),
                join_tbl_on_condition(from_tbl, to_tbl, rel.from_col, rel.to_col),
            );
        }
        slf = slf.apply_alias(SelectA.as_str());
        let mut select_two = SelectTwo::new_without_prepare(slf.query);
        let tbl = Alias::new(&format!("r{}", l.link().len() - 1));
        for col in <T::Column as Iterable>::iter() {
            let alias = format!("{}{}", SelectB.as_str(), col.as_str());
            select_two.query().expr(SelectExpr {
                expr: Expr::tbl(tbl.clone(), col).into(),
                alias: Some(SeaRc::new(Alias::new(&alias))),
            });
        }
        match <<T as EntityTrait>::Model as ModelTrait>::soft_delete_column() {
            Some(soft_delete_column) if !with_deleted => {
                select_two
                    .query()
                    .and_where(Expr::tbl(tbl, soft_delete_column).is_null());
            }
            _ => {}
        }
        select_two
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_cfg::{cake, cake_filling, cake_filling_price, entity_linked, filling, fruit};
    use crate::{ColumnTrait, DbBackend, EntityTrait, ModelTrait, QueryFilter, QueryTrait};
    use pretty_assertions::assert_eq;

    #[test]
    fn join_1() {
        assert_eq!(
            cake::Entity::find()
                .left_join(fruit::Entity)
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `cake`.`id`, `cake`.`name` FROM `cake`",
                "LEFT JOIN `fruit` ON `cake`.`id` = `fruit`.`cake_id`",
            ]
            .join(" ")
        );
    }

    #[test]
    fn join_2() {
        assert_eq!(
            cake::Entity::find()
                .inner_join(fruit::Entity)
                .filter(fruit::Column::Name.contains("cherry"))
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `cake`.`id`, `cake`.`name` FROM `cake`",
                "INNER JOIN `fruit` ON `cake`.`id` = `fruit`.`cake_id`",
                "WHERE `fruit`.`name` LIKE \'%cherry%\'"
            ]
            .join(" ")
        );
    }

    #[test]
    fn join_3() {
        assert_eq!(
            fruit::Entity::find()
                .reverse_join(cake::Entity)
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit`",
                "INNER JOIN `cake` ON `cake`.`id` = `fruit`.`cake_id`",
            ]
            .join(" ")
        );
    }

    #[test]
    fn join_4() {
        use crate::{Related, Select};

        let find_fruit: Select<fruit::Entity> = cake::Entity::find_related();
        assert_eq!(
            find_fruit
                .filter(cake::Column::Id.eq(11))
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit`",
                "INNER JOIN `cake` ON `cake`.`id` = `fruit`.`cake_id`",
                "WHERE `cake`.`id` = 11",
            ]
            .join(" ")
        );
    }

    #[test]
    fn join_5() {
        let cake_model = cake::Model {
            id: 12,
            name: "".to_owned(),
        };

        assert_eq!(
            cake_model
                .find_related(fruit::Entity)
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit`",
                "INNER JOIN `cake` ON `cake`.`id` = `fruit`.`cake_id`",
                "WHERE `cake`.`id` = 12",
            ]
            .join(" ")
        );
    }

    #[test]
    fn join_6() {
        assert_eq!(
            cake::Entity::find()
                .left_join(filling::Entity)
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `cake`.`id`, `cake`.`name` FROM `cake`",
                "LEFT JOIN `cake_filling` ON `cake`.`id` = `cake_filling`.`cake_id`",
                "LEFT JOIN `filling` ON `cake_filling`.`filling_id` = `filling`.`id`",
            ]
            .join(" ")
        );
    }

    #[test]
    fn join_7() {
        use crate::{Related, Select};

        let find_filling: Select<filling::Entity> = cake::Entity::find_related();
        assert_eq!(
            find_filling.build(DbBackend::MySql).to_string(),
            [
                "SELECT `filling`.`id`, `filling`.`name`, `filling`.`vendor_id` FROM `filling`",
                "INNER JOIN `cake_filling` ON `cake_filling`.`filling_id` = `filling`.`id`",
                "INNER JOIN `cake` ON `cake`.`id` = `cake_filling`.`cake_id`",
            ]
            .join(" ")
        );
    }

    #[test]
    fn join_8() {
        use crate::{Related, Select};

        let find_cake_filling_price: Select<cake_filling_price::Entity> =
            cake_filling::Entity::find_related();
        assert_eq!(
            find_cake_filling_price.build(DbBackend::Postgres).to_string(),
            [
                r#"SELECT "cake_filling_price"."cake_id", "cake_filling_price"."filling_id", "cake_filling_price"."price""#,
                r#"FROM "public"."cake_filling_price""#,
                r#"INNER JOIN "cake_filling" ON"#,
                r#"("cake_filling"."cake_id" = "cake_filling_price"."cake_id") AND"#,
                r#"("cake_filling"."filling_id" = "cake_filling_price"."filling_id")"#,
            ]
            .join(" ")
        );
    }

    #[test]
    fn join_9() {
        use crate::{Related, Select};

        let find_cake_filling: Select<cake_filling::Entity> =
            cake_filling_price::Entity::find_related();
        assert_eq!(
            find_cake_filling.build(DbBackend::Postgres).to_string(),
            [
                r#"SELECT "cake_filling"."cake_id", "cake_filling"."filling_id""#,
                r#"FROM "cake_filling""#,
                r#"INNER JOIN "public"."cake_filling_price" ON"#,
                r#"("cake_filling_price"."cake_id" = "cake_filling"."cake_id") AND"#,
                r#"("cake_filling_price"."filling_id" = "cake_filling"."filling_id")"#,
            ]
            .join(" ")
        );
    }

    #[test]
    fn join_10() {
        let cake_model = cake::Model {
            id: 12,
            name: "".to_owned(),
        };

        assert_eq!(
            cake_model
                .find_linked(entity_linked::CakeToFilling)
                .build(DbBackend::MySql)
                .to_string(),
            [
                r#"SELECT `filling`.`id`, `filling`.`name`, `filling`.`vendor_id`"#,
                r#"FROM `filling`"#,
                r#"INNER JOIN `cake_filling` AS `r0` ON `r0`.`filling_id` = `filling`.`id`"#,
                r#"INNER JOIN `cake` AS `r1` ON `r1`.`id` = `r0`.`cake_id`"#,
                r#"WHERE `r1`.`id` = 12"#,
            ]
            .join(" ")
        );
    }

    #[test]
    fn join_11() {
        let cake_model = cake::Model {
            id: 18,
            name: "".to_owned(),
        };

        assert_eq!(
            cake_model
                .find_linked(entity_linked::CakeToFillingVendor)
                .build(DbBackend::MySql)
                .to_string(),
            [
                r#"SELECT `vendor`.`id`, `vendor`.`name`"#,
                r#"FROM `vendor`"#,
                r#"INNER JOIN `filling` AS `r0` ON `r0`.`vendor_id` = `vendor`.`id`"#,
                r#"INNER JOIN `cake_filling` AS `r1` ON `r1`.`filling_id` = `r0`.`id`"#,
                r#"INNER JOIN `cake` AS `r2` ON `r2`.`id` = `r1`.`cake_id`"#,
                r#"WHERE `r2`.`id` = 18"#,
            ]
            .join(" ")
        );
    }

    #[test]
    fn join_12() {
        assert_eq!(
            cake::Entity::find()
                .find_also_linked(entity_linked::CakeToFilling)
                .build(DbBackend::MySql)
                .to_string(),
            [
                r#"SELECT `cake`.`id` AS `A_id`, `cake`.`name` AS `A_name`,"#,
                r#"`r1`.`id` AS `B_id`, `r1`.`name` AS `B_name`, `r1`.`vendor_id` AS `B_vendor_id`"#,
                r#"FROM `cake`"#,
                r#"LEFT JOIN `cake_filling` AS `r0` ON `cake`.`id` = `r0`.`cake_id`"#,
                r#"LEFT JOIN `filling` AS `r1` ON `r0`.`filling_id` = `r1`.`id`"#,
            ]
            .join(" ")
        );
    }

    #[test]
    fn join_13() {
        assert_eq!(
            cake::Entity::find()
                .find_also_linked(entity_linked::CakeToFillingVendor)
                .build(DbBackend::MySql)
                .to_string(),
            [
                r#"SELECT `cake`.`id` AS `A_id`, `cake`.`name` AS `A_name`,"#,
                r#"`r2`.`id` AS `B_id`, `r2`.`name` AS `B_name`"#,
                r#"FROM `cake`"#,
                r#"LEFT JOIN `cake_filling` AS `r0` ON `cake`.`id` = `r0`.`cake_id`"#,
                r#"LEFT JOIN `filling` AS `r1` ON `r0`.`filling_id` = `r1`.`id`"#,
                r#"LEFT JOIN `vendor` AS `r2` ON `r1`.`vendor_id` = `r2`.`id`"#,
            ]
            .join(" ")
        );
    }
}
