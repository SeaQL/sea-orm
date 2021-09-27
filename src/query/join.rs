use crate::{EntityTrait, Linked, QuerySelect, Related, Select, SelectTwo, SelectTwoMany};
pub use sea_query::JoinType;

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
        self.join_join(JoinType::LeftJoin, E::to(), E::via())
    }

    /// Right Join with a Related Entity.
    pub fn right_join<R>(self, _: R) -> Self
    where
        R: EntityTrait,
        E: Related<R>,
    {
        self.join_join(JoinType::RightJoin, E::to(), E::via())
    }

    /// Inner Join with a Related Entity.
    pub fn inner_join<R>(self, _: R) -> Self
    where
        R: EntityTrait,
        E: Related<R>,
    {
        self.join_join(JoinType::InnerJoin, E::to(), E::via())
    }

    /// Join with an Entity Related to me.
    pub fn reverse_join<R>(self, _: R) -> Self
    where
        R: EntityTrait + Related<E>,
    {
        self.join_rev(JoinType::InnerJoin, R::to())
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
        let mut slf = self;
        for rel in l.link() {
            slf = slf.join(JoinType::LeftJoin, rel);
        }
        slf.select_also(T::default())
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
                r#"`filling`.`id` AS `B_id`, `filling`.`name` AS `B_name`, `filling`.`vendor_id` AS `B_vendor_id`"#,
                r#"FROM `cake`"#,
                r#"LEFT JOIN `cake_filling` ON `cake`.`id` = `cake_filling`.`cake_id`"#,
                r#"LEFT JOIN `filling` ON `cake_filling`.`filling_id` = `filling`.`id`"#,
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
                r#"`vendor`.`id` AS `B_id`, `vendor`.`name` AS `B_name`"#,
                r#"FROM `cake`"#,
                r#"LEFT JOIN `cake_filling` ON `cake`.`id` = `cake_filling`.`cake_id`"#,
                r#"LEFT JOIN `filling` ON `cake_filling`.`filling_id` = `filling`.`id`"#,
                r#"LEFT JOIN `vendor` ON `filling`.`vendor_id` = `vendor`.`id`"#,
            ]
            .join(" ")
        );
    }
}
