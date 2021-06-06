use crate::{
    ActiveModelTrait, ColumnTrait, EntityTrait, Iterable, PrimaryKeyToColumn, QueryFilter,
    QueryTrait,
};
use sea_query::{IntoIden, UpdateStatement};

#[derive(Clone, Debug)]
pub struct Update<A>
where
    A: ActiveModelTrait,
{
    pub(crate) query: UpdateStatement,
    pub(crate) model: A,
}

impl<A> Update<A>
where
    A: ActiveModelTrait,
{
    pub fn new<M>(model: M) -> Self
    where
        M: Into<A>,
    {
        let myself = Self {
            query: UpdateStatement::new()
                .table(A::Entity::default().into_iden())
                .to_owned(),
            model: model.into(),
        };
        myself.prepare()
    }

    pub(crate) fn prepare(mut self) -> Self {
        for key in <A::Entity as EntityTrait>::PrimaryKey::iter() {
            let col = key.into_column();
            let av = self.model.get(col);
            if av.is_set() || av.is_unchanged() {
                self = self.filter(col.eq(av.unwrap()));
            } else {
                panic!("PrimaryKey is not set");
            }
        }
        for col in <A::Entity as EntityTrait>::Column::iter() {
            if <A::Entity as EntityTrait>::PrimaryKey::from_column(col).is_some() {
                continue;
            }
            let av = self.model.get(col);
            if av.is_set() {
                self.query.value(col, av.unwrap());
            }
        }
        self
    }
}

impl<A> QueryFilter for Update<A>
where
    A: ActiveModelTrait,
{
    type QueryStatement = UpdateStatement;

    fn query(&mut self) -> &mut UpdateStatement {
        &mut self.query
    }
}

impl<A> QueryTrait for Update<A>
where
    A: ActiveModelTrait,
{
    type QueryStatement = UpdateStatement;

    fn query(&mut self) -> &mut UpdateStatement {
        &mut self.query
    }

    fn as_query(&self) -> &UpdateStatement {
        &self.query
    }

    fn into_query(self) -> UpdateStatement {
        self.query
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_cfg::{cake, fruit};
    use crate::{ActiveValue, QueryTrait, Update};
    use sea_query::PostgresQueryBuilder;

    #[test]
    fn update_1() {
        assert_eq!(
            Update::<cake::ActiveModel>::new(cake::ActiveModel {
                id: ActiveValue::set(1),
                name: ActiveValue::set("Apple Pie".to_owned()),
            })
            .build(PostgresQueryBuilder)
            .to_string(),
            r#"UPDATE "cake" SET "name" = 'Apple Pie' WHERE "cake"."id" = 1"#,
        );
    }

    #[test]
    fn update_2() {
        assert_eq!(
            Update::<fruit::ActiveModel>::new(fruit::ActiveModel {
                id: ActiveValue::set(1),
                name: ActiveValue::set("Orange".to_owned()),
                cake_id: ActiveValue::unset(),
            })
            .build(PostgresQueryBuilder)
            .to_string(),
            r#"UPDATE "fruit" SET "name" = 'Orange' WHERE "fruit"."id" = 1"#,
        );
    }

    #[test]
    fn update_3() {
        assert_eq!(
            Update::<fruit::ActiveModel>::new(fruit::ActiveModel {
                id: ActiveValue::set(2),
                name: ActiveValue::unchanged("Apple".to_owned()),
                cake_id: ActiveValue::set(Some(3)),
            })
            .build(PostgresQueryBuilder)
            .to_string(),
            r#"UPDATE "fruit" SET "cake_id" = 3 WHERE "fruit"."id" = 2"#,
        );
    }
}
