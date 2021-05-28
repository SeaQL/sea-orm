use crate::{ActiveModelOf, ActiveModelTrait, EntityTrait, Iterable, Statement};
use core::marker::PhantomData;
use sea_query::{InsertStatement, IntoIden, QueryBuilder};

#[derive(Clone, Debug)]
pub struct Insert<A>
where
    A: ActiveModelTrait,
{
    pub(crate) query: InsertStatement,
    pub(crate) model: PhantomData<A>,
}

impl<A> Insert<A>
where
    A: ActiveModelTrait,
{
    pub fn new<E>() -> Self
    where
        E: EntityTrait,
        A: ActiveModelOf<E>,
    {
        Self {
            query: InsertStatement::new()
                .into_table(E::default().into_iden())
                .to_owned(),
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
        for col in A::Column::iter() {
            let av = am.take(col);
            if av.is_set() {
                columns.push(col);
                values.push(av.into_value());
            }
        }
        self.query.columns(columns);
        self.query.values_panic(values);
        self
    }

    /// Get a mutable ref to the query builder
    pub fn query(&mut self) -> &mut InsertStatement {
        &mut self.query
    }

    /// Get an immutable ref to the query builder
    pub fn as_query(&self) -> &InsertStatement {
        &self.query
    }

    /// Take ownership of the query builder
    pub fn into_query(self) -> InsertStatement {
        self.query
    }

    /// Build the query as [`Statement`]
    pub fn build<B>(&self, builder: B) -> Statement
    where
        B: QueryBuilder,
    {
        self.as_query().build(builder).into()
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_cfg::cake;
    use crate::{ActiveValue, Insert};
    use sea_query::PostgresQueryBuilder;

    #[test]
    fn insert_1() {
        assert_eq!(
            Insert::<cake::ActiveModel>::new()
                .one(cake::ActiveModel {
                    id: ActiveValue::unset(),
                    name: ActiveValue::set("Apple Pie".to_owned()),
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
                    id: ActiveValue::set(1),
                    name: ActiveValue::set("Apple Pie".to_owned()),
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
}
