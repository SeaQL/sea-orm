use crate::{ActiveModelTrait, EntityTrait, Iterable, QueryTrait};
use core::marker::PhantomData;
use sea_query::{InsertStatement, IntoIden};

#[derive(Clone, Debug)]
pub struct Insert<A>
where
    A: ActiveModelTrait,
{
    pub(crate) query: InsertStatement,
    pub(crate) columns: Vec<bool>,
    pub(crate) model: PhantomData<A>,
}

impl<A> Default for Insert<A>
where
    A: ActiveModelTrait,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<A> Insert<A>
where
    A: ActiveModelTrait,
{
    pub fn new() -> Self {
        Self {
            query: InsertStatement::new()
                .into_table(A::Entity::default().into_iden())
                .to_owned(),
            columns: Vec::new(),
            model: PhantomData,
        }
    }

    pub fn one<M>(mut self, m: M) -> Self
    where
        M: Into<A>,
    {
        let mut am: A = m.into();
        let mut columns = Vec::new();
        let mut values = Vec::new();
        let columns_empty = self.columns.is_empty();
        for (idx, col) in <A::Entity as EntityTrait>::Column::iter().enumerate() {
            let av = am.take(col);
            if columns_empty {
                self.columns.push(av.is_set());
            } else if self.columns[idx] != av.is_set() {
                panic!("columns mismatch");
            }
            if av.is_set() {
                columns.push(col);
                values.push(av.into_value());
            }
        }
        self.query.columns(columns);
        self.query.values_panic(values);
        self
    }

    pub fn many<M, I>(mut self, models: I) -> Self
    where
        M: Into<A>,
        I: IntoIterator<Item = M>,
    {
        for model in models.into_iter() {
            self = self.one(model);
        }
        self
    }
}

impl<A> QueryTrait for Insert<A>
where
    A: ActiveModelTrait,
{
    type QueryStatement = InsertStatement;

    fn query(&mut self) -> &mut InsertStatement {
        &mut self.query
    }

    fn as_query(&self) -> &InsertStatement {
        &self.query
    }

    fn into_query(self) -> InsertStatement {
        self.query
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_cfg::cake;
    use crate::{Insert, QueryTrait, Val};
    use sea_query::PostgresQueryBuilder;

    #[test]
    fn insert_1() {
        assert_eq!(
            Insert::<cake::ActiveModel>::new()
                .one(cake::ActiveModel {
                    id: Val::unset(),
                    name: Val::set("Apple Pie".to_owned()),
                })
                .build(PostgresQueryBuilder)
                .to_string(),
            r#"INSERT INTO "cake" ("name") VALUES ('Apple Pie')"#,
        );
    }

    #[test]
    fn insert_2() {
        assert_eq!(
            Insert::<cake::ActiveModel>::new()
                .one(cake::ActiveModel {
                    id: Val::set(1),
                    name: Val::set("Apple Pie".to_owned()),
                })
                .build(PostgresQueryBuilder)
                .to_string(),
            r#"INSERT INTO "cake" ("id", "name") VALUES (1, 'Apple Pie')"#,
        );
    }

    #[test]
    fn insert_3() {
        assert_eq!(
            Insert::<cake::ActiveModel>::new()
                .one(cake::Model {
                    id: 1,
                    name: "Apple Pie".to_owned(),
                })
                .build(PostgresQueryBuilder)
                .to_string(),
            r#"INSERT INTO "cake" ("id", "name") VALUES (1, 'Apple Pie')"#,
        );
    }

    #[test]
    fn insert_4() {
        assert_eq!(
            Insert::<cake::ActiveModel>::new()
                .many(vec![
                    cake::Model {
                        id: 1,
                        name: "Apple Pie".to_owned(),
                    },
                    cake::Model {
                        id: 2,
                        name: "Orange Scone".to_owned(),
                    }
                ])
                .build(PostgresQueryBuilder)
                .to_string(),
            r#"INSERT INTO "cake" ("id", "name") VALUES (1, 'Apple Pie'), (2, 'Orange Scone')"#,
        );
    }

    #[test]
    #[should_panic(expected = "columns mismatch")]
    fn insert_5() {
        let apple = cake::ActiveModel {
            name: Val::set("Apple".to_owned()),
            ..Default::default()
        };
        let orange = cake::ActiveModel {
            id: Val::set(2),
            name: Val::set("Orange".to_owned()),
        };
        assert_eq!(
            Insert::<cake::ActiveModel>::new()
                .many(vec![apple, orange])
                .build(PostgresQueryBuilder)
                .to_string(),
            r#"INSERT INTO "cake" ("id", "name") VALUES (NULL, 'Apple'), (2, 'Orange')"#,
        );
    }
}
