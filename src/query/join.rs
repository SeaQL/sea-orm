use crate::{EntityTrait, QuerySelect, Related, Select, SelectTwo};
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
    pub fn left_join_and_select_also<R>(self, r: R) -> SelectTwo<E, R>
    where
        R: EntityTrait,
        E: Related<R>,
    {
        self.left_join(r).select_also(r)
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_cfg::{cake, filling, fruit};
    use crate::{ColumnTrait, EntityTrait, QueryFilter, QueryTrait};
    use sea_query::MysqlQueryBuilder;

    #[test]
    fn join_1() {
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
    fn join_2() {
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
    fn join_3() {
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
    fn join_4() {
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
    fn join_5() {
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
    fn join_6() {
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
    fn join_7() {
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
}
