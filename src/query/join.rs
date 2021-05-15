use crate::{
    ColumnTrait, EntityTrait, Identity, Iterable, ModelTrait, PrimaryKeyOfModel, QueryHelper,
    Related, RelationDef, Select,
};

pub use sea_query::JoinType;
use sea_query::{Expr, IntoIden};
use std::rc::Rc;

impl<E> Select<E>
where
    E: EntityTrait,
{
    pub fn belongs_to<R>(self, model: &R::Model) -> Self
    where
        R: EntityTrait + Related<E>,
        R::PrimaryKey: PrimaryKeyOfModel<R::Model>,
    {
        if let Some(key) = R::PrimaryKey::iter().next() {
            // TODO: supporting composite primary key
            let col = key.into_column();
            self.filter(col.eq(model.get(col)))
        } else {
            panic!("undefined primary key");
        }
    }

    /// Join via [`RelationDef`].
    pub fn join(mut self, join: JoinType, rel: RelationDef) -> Self {
        let own_tbl = E::default().into_iden();
        let to_tbl = rel.to_tbl.clone();
        let owner_keys = rel.from_col;
        let foreign_keys = rel.to_col;
        let condition = match (owner_keys, foreign_keys) {
            (Identity::Unary(o1), Identity::Unary(f1)) => {
                Expr::tbl(Rc::clone(&own_tbl), o1).equals(Rc::clone(&to_tbl), f1)
            } // _ => panic!("Owner key and foreign key mismatch"),
        };
        self.query.join(join, Rc::clone(&to_tbl), condition);
        self
    }

    /// Join via [`RelationDef`] but in reverse direction.
    /// Assume when there exist a relation A -> B.
    /// You can reverse join B <- A.
    pub fn join_rev(mut self, join: JoinType, rel: RelationDef) -> Self {
        let from_tbl = rel.from_tbl.clone();
        let to_tbl = rel.to_tbl.clone();
        let owner_keys = rel.from_col;
        let foreign_keys = rel.to_col;
        let condition = match (owner_keys, foreign_keys) {
            (Identity::Unary(o1), Identity::Unary(f1)) => {
                Expr::tbl(Rc::clone(&from_tbl), o1).equals(Rc::clone(&to_tbl), f1)
            } // _ => panic!("Owner key and foreign key mismatch"),
        };
        self.query.join(join, Rc::clone(&from_tbl), condition);
        self
    }

    /// Left Join with a Related Entity.
    pub fn left_join<R>(self, _: R) -> Self
    where
        R: EntityTrait,
        E: Related<R>,
    {
        self.join(JoinType::LeftJoin, E::to())
    }

    /// Right Join with a Related Entity.
    pub fn right_join<R>(self, _: R) -> Self
    where
        R: EntityTrait,
        E: Related<R>,
    {
        self.join(JoinType::RightJoin, E::to())
    }

    /// Inner Join with a Related Entity.
    pub fn inner_join<R>(self, _: R) -> Self
    where
        R: EntityTrait,
        E: Related<R>,
    {
        self.join(JoinType::InnerJoin, E::to())
    }

    /// Join with an Entity Related to me.
    pub fn reverse_join<R>(self, _: R) -> Self
    where
        R: EntityTrait + Related<E>,
    {
        self.join_rev(JoinType::InnerJoin, R::to())
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_cfg::{cake, fruit};
    use crate::{ColumnTrait, EntityTrait, QueryHelper};
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
    fn alias_1() {
        assert_eq!(
            cake::Entity::find()
                .column_as(cake::Column::Id, "B")
                .apply_alias("A_")
                .build(MysqlQueryBuilder)
                .to_string(),
            "SELECT `cake`.`id` AS `A_id`, `cake`.`name` AS `A_name`, `cake`.`id` AS `A_B` FROM `cake`",
        );
    }
}
