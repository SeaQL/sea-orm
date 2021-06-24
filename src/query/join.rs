use crate::{EntityTrait, QuerySelect, Related, Select, SelectThree, SelectTwo};
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
    pub fn left_join_and_select<R>(self, r: R) -> SelectTwo<E, R>
    where
        R: EntityTrait,
        E: Related<R>,
    {
        self.left_join(r).select_also(r)
    }
}

impl<E, F> SelectTwo<E, F>
where
    E: EntityTrait,
    F: EntityTrait,
{
    /// Left Join with a Related Entity.
    pub fn left_join<R>(self, _: R) -> Self
    where
        R: EntityTrait,
        F: Related<R>,
    {
        self.join_join(JoinType::LeftJoin, F::to(), F::via())
    }

    /// Right Join with a Related Entity.
    pub fn right_join<R>(self, _: R) -> Self
    where
        R: EntityTrait,
        F: Related<R>,
    {
        self.join_join(JoinType::RightJoin, F::to(), F::via())
    }

    /// Inner Join with a Related Entity.
    pub fn inner_join<R>(self, _: R) -> Self
    where
        R: EntityTrait,
        F: Related<R>,
    {
        self.join_join(JoinType::InnerJoin, F::to(), F::via())
    }

    /// Join with an Entity Related to me.
    pub fn reverse_join<R>(self, _: R) -> Self
    where
        R: EntityTrait + Related<E>,
    {
        self.join_rev(JoinType::InnerJoin, R::to())
    }

    /// Left Join with a Related Entity and select both Entity.
    pub fn left_join_and_select<R>(self, r: R) -> SelectThree<E, F, R>
    where
        R: EntityTrait,
        F: Related<R>,
    {
        self.left_join(r).select_also(r)
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_cfg::{cake, filling, fruit, vendor};
    use crate::{ColumnTrait, EntityTrait, QueryFilter, QueryTrait};
    use sea_query::MysqlQueryBuilder;

    #[test]
    fn select_join_1() {
        assert_eq!(
            cake::Entity::find()
                .left_join(fruit::Entity)
                .build(MysqlQueryBuilder)
                .to_string(),
            [
                "SELECT `cake`.`id`, `cake`.`name` FROM `cake`",
                "LEFT JOIN `fruit` ON `cake`.`id` = `fruit`.`cake_id`",
            ]
            .join(" ")
        );
    }

    #[test]
    fn select_join_2() {
        assert_eq!(
            cake::Entity::find()
                .inner_join(fruit::Entity)
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

    #[test]
    fn select_join_3() {
        assert_eq!(
            fruit::Entity::find()
                .reverse_join(cake::Entity)
                .build(MysqlQueryBuilder)
                .to_string(),
            [
                "SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit`",
                "INNER JOIN `cake` ON `cake`.`id` = `fruit`.`cake_id`",
            ]
            .join(" ")
        );
    }

    #[test]
    fn select_join_4() {
        use crate::{Related, Select};

        let find_fruit: Select<fruit::Entity> = cake::Entity::find_related();
        assert_eq!(
            find_fruit
                .filter(cake::Column::Id.eq(11))
                .build(MysqlQueryBuilder)
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
    fn select_join_5() {
        let cake_model = cake::Model {
            id: 12,
            name: "".to_owned(),
        };

        assert_eq!(
            cake_model.find_fruit().build(MysqlQueryBuilder).to_string(),
            [
                "SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit`",
                "INNER JOIN `cake` ON `cake`.`id` = `fruit`.`cake_id`",
                "WHERE `cake`.`id` = 12",
            ]
            .join(" ")
        );
    }

    #[test]
    fn select_join_6() {
        assert_eq!(
            cake::Entity::find()
                .left_join(filling::Entity)
                .build(MysqlQueryBuilder)
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
    fn select_join_7() {
        use crate::{Related, Select};

        let find_filling: Select<filling::Entity> = cake::Entity::find_related();
        assert_eq!(
            find_filling.build(MysqlQueryBuilder).to_string(),
            [
                "SELECT `filling`.`id`, `filling`.`name` FROM `filling`",
                "INNER JOIN `cake_filling` ON `cake_filling`.`filling_id` = `filling`.`id`",
                "INNER JOIN `cake` ON `cake`.`id` = `cake_filling`.`cake_id`",
            ]
            .join(" ")
        );
    }

    #[test]
    fn select_left_join_and_select_1() {
        assert_eq!(
            cake::Entity::find()
                .left_join_and_select(fruit::Entity)
                .build(MysqlQueryBuilder)
                .to_string(),
            cake::Entity::find()
                .left_join(fruit::Entity)
                .select_also(fruit::Entity)
                .build(MysqlQueryBuilder)
                .to_string(),
        );
    }

    #[test]
    fn select_two_join_1() {
        assert_eq!(
            cake::Entity::find()
                .left_join(fruit::Entity)
                .select_also(fruit::Entity)
                .left_join(vendor::Entity)
                .build(MysqlQueryBuilder)
                .to_string(),
            [
                "SELECT `cake`.`id` AS `A_id`, `cake`.`name` AS `A_name`,",
                "`fruit`.`id` AS `B_id`, `fruit`.`name` AS `B_name`, `fruit`.`cake_id` AS `B_cake_id`",
                "FROM `cake`",
                "LEFT JOIN `fruit` ON `cake`.`id` = `fruit`.`cake_id`",
                "LEFT JOIN `vendor` ON `fruit`.`id` = `vendor`.`fruit_id`",
                "ORDER BY `cake`.`id` ASC",
            ]
            .join(" ")
        );
    }

    #[test]
    fn select_two_join_2() {
        assert_eq!(
            cake::Entity::find()
                .inner_join(fruit::Entity)
                .select_also(fruit::Entity)
                .filter(fruit::Column::Name.contains("cherry"))
                .inner_join(vendor::Entity)
                .build(MysqlQueryBuilder)
                .to_string(),
            [
                "SELECT `cake`.`id` AS `A_id`, `cake`.`name` AS `A_name`,",
                "`fruit`.`id` AS `B_id`, `fruit`.`name` AS `B_name`, `fruit`.`cake_id` AS `B_cake_id`",
                "FROM `cake`",
                "INNER JOIN `fruit` ON `cake`.`id` = `fruit`.`cake_id`",
                "INNER JOIN `vendor` ON `fruit`.`id` = `vendor`.`fruit_id`",
                "WHERE `fruit`.`name` LIKE \'%cherry%\'",
                "ORDER BY `cake`.`id` ASC",
            ]
            .join(" ")
        );
    }

    #[test]
    fn select_two_join_3() {
        assert_eq!(
            fruit::Entity::find()
                .inner_join(cake::Entity)
                .select_also(cake::Entity)
                .reverse_join(vendor::Entity)
                .build(MysqlQueryBuilder)
                .to_string(),
            [
                "SELECT `fruit`.`id` AS `A_id`, `fruit`.`name` AS `A_name`, `fruit`.`cake_id` AS `A_cake_id`,",
                "`cake`.`id` AS `B_id`, `cake`.`name` AS `B_name`",
                "FROM `fruit`",
                "INNER JOIN `cake` ON `fruit`.`cake_id` = `cake`.`id`",
                "INNER JOIN `vendor` ON `vendor`.`fruit_id` = `fruit`.`id`",
                "ORDER BY `fruit`.`id` ASC",
            ]
            .join(" ")
        );
    }

    #[test]
    fn select_two_join_4() {
        use crate::{Related, Select};

        let find_fruit: Select<fruit::Entity> = cake::Entity::find_related();
        let find_fruit_vendor = find_fruit
            .inner_join(vendor::Entity)
            .select_also(vendor::Entity);
        assert_eq!(
            find_fruit_vendor
                .filter(cake::Column::Id.eq(11))
                .build(MysqlQueryBuilder)
                .to_string(),
            [
                "SELECT `fruit`.`id` AS `A_id`, `fruit`.`name` AS `A_name`, `fruit`.`cake_id` AS `A_cake_id`,",
                "`vendor`.`id` AS `B_id`, `vendor`.`name` AS `B_name`, `vendor`.`fruit_id` AS `B_fruit_id`",
                "FROM `fruit`",
                "INNER JOIN `cake` ON `cake`.`id` = `fruit`.`cake_id`",
                "INNER JOIN `vendor` ON `fruit`.`id` = `vendor`.`fruit_id`",
                "WHERE `cake`.`id` = 11",
                "ORDER BY `fruit`.`id` ASC",
            ]
            .join(" ")
        );
    }

    #[test]
    fn select_two_join_5() {
        let cake_model = cake::Model {
            id: 12,
            name: "".to_owned(),
        };
        let find_fruit = cake_model.find_fruit();
        let find_fruit_vendor = find_fruit
            .inner_join(vendor::Entity)
            .select_also(vendor::Entity);

        assert_eq!(
            find_fruit_vendor.build(MysqlQueryBuilder).to_string(),
            [
                "SELECT `fruit`.`id` AS `A_id`, `fruit`.`name` AS `A_name`, `fruit`.`cake_id` AS `A_cake_id`,",
                "`vendor`.`id` AS `B_id`, `vendor`.`name` AS `B_name`, `vendor`.`fruit_id` AS `B_fruit_id`",
                "FROM `fruit`",
                "INNER JOIN `cake` ON `cake`.`id` = `fruit`.`cake_id`",
                "INNER JOIN `vendor` ON `fruit`.`id` = `vendor`.`fruit_id`",
                "WHERE `cake`.`id` = 12",
                "ORDER BY `fruit`.`id` ASC",
            ]
            .join(" ")
        );
    }

    #[test]
    fn select_two_left_join_and_select_1() {
        assert_eq!(
            cake::Entity::find()
                .left_join_and_select(fruit::Entity)
                .left_join_and_select(vendor::Entity)
                .build(MysqlQueryBuilder)
                .to_string(),
            cake::Entity::find()
                .left_join(fruit::Entity)
                .select_also(fruit::Entity)
                .left_join(vendor::Entity)
                .select_also(vendor::Entity)
                .build(MysqlQueryBuilder)
                .to_string(),
        );
    }
}
