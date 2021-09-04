use crate::{
    ActiveModelTrait, ActiveValue, EntityName, EntityTrait, IntoActiveModel, Iterable, QueryTrait,
};
use core::marker::PhantomData;
use sea_query::{InsertStatement, Value};

pub trait Insertable {
    type Entity: EntityTrait;

    fn take(&mut self, c: <Self::Entity as EntityTrait>::Column) -> ActiveValue<Value>;
}

impl<T, E> Insertable for T
where
    T: ActiveModelTrait<Entity = E>,
    E: EntityTrait,
{
    type Entity = E;

    fn take(&mut self, c: <Self::Entity as EntityTrait>::Column) -> ActiveValue<Value> {
        self.take(c)
    }
}

pub trait IntoInsertable<A>
where
    A: Insertable,
{
    fn into_insertable(self) -> A;
}

impl<A> IntoInsertable<A> for A
where
    A: Insertable,
{
    fn into_insertable(self) -> A {
        self
    }
}

#[derive(Clone, Debug)]
pub struct Insert<A>
where
    A: Insertable,
{
    pub(crate) query: InsertStatement,
    pub(crate) columns: Vec<bool>,
    pub(crate) model: PhantomData<A>,
}

impl<A> Default for Insert<A>
where
    A: Insertable,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<A> Insert<A>
where
    A: Insertable,
{
    pub(crate) fn new() -> Self {
        Self {
            query: InsertStatement::new()
                .into_table(A::Entity::default().table_ref())
                .to_owned(),
            columns: Vec::new(),
            model: PhantomData,
        }
    }

    /// Insert one Model or ActiveModel
    ///
    /// Model
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     Insert::one(cake::Model {
    ///         id: 1,
    ///         name: "Apple Pie".to_owned(),
    ///     })
    ///     .build(DbBackend::Postgres)
    ///     .to_string(),
    ///     r#"INSERT INTO "cake" ("id", "name") VALUES (1, 'Apple Pie')"#,
    /// );
    /// ```
    /// ActiveModel
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     Insert::one(cake::ActiveModel {
    ///         id: Unset(None),
    ///         name: Set("Apple Pie".to_owned()),
    ///     })
    ///     .build(DbBackend::Postgres)
    ///     .to_string(),
    ///     r#"INSERT INTO "cake" ("name") VALUES ('Apple Pie')"#,
    /// );
    /// ```
    pub fn one<M>(m: M) -> Insert<A>
    where
        M: IntoInsertable<A>,
    {
        Self::new().add(m)
    }

    /// Insert many Model or ActiveModel
    ///
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     Insert::many(vec![
    ///         cake::Model {
    ///             id: 1,
    ///             name: "Apple Pie".to_owned(),
    ///         },
    ///         cake::Model {
    ///             id: 2,
    ///             name: "Orange Scone".to_owned(),
    ///         }
    ///     ])
    ///     .build(DbBackend::Postgres)
    ///     .to_string(),
    ///     r#"INSERT INTO "cake" ("id", "name") VALUES (1, 'Apple Pie'), (2, 'Orange Scone')"#,
    /// );
    /// ```
    pub fn many<M, I>(models: I) -> Self
    where
        M: IntoInsertable<A>,
        I: IntoIterator<Item = M>,
    {
        Self::new().add_many(models)
    }

    #[allow(clippy::should_implement_trait)]
    pub fn add<M>(mut self, m: M) -> Self
    where
        M: IntoInsertable<A>,
    {
        let mut am: A = m.into_insertable();
        let mut columns = Vec::new();
        let mut values = Vec::new();
        let columns_empty = self.columns.is_empty();
        for (idx, col) in <A::Entity as EntityTrait>::Column::iter().enumerate() {
            let av = am.take(col);
            let av_has_val = av.is_set() || av.is_unchanged();
            if columns_empty {
                self.columns.push(av_has_val);
            } else if self.columns[idx] != av_has_val {
                panic!("columns mismatch");
            }
            if av_has_val {
                columns.push(col);
                values.push(av.into_value());
            }
        }
        self.query.columns(columns);
        self.query.values_panic(values);
        self
    }

    pub fn add_many<M, I>(mut self, models: I) -> Self
    where
        M: IntoInsertable<A>,
        I: IntoIterator<Item = M>,
    {
        for model in models.into_iter() {
            self = self.add(model);
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
    use crate::{ActiveValue, DbBackend, Insert, QueryTrait};

    #[test]
    fn insert_1() {
        assert_eq!(
            Insert::<cake::ActiveModel>::new()
                .add(cake::ActiveModel {
                    id: ActiveValue::unset(),
                    name: ActiveValue::set("Apple Pie".to_owned()),
                })
                .build(DbBackend::Postgres)
                .to_string(),
            r#"INSERT INTO "cake" ("name") VALUES ('Apple Pie')"#,
        );
    }

    #[test]
    fn insert_2() {
        assert_eq!(
            Insert::<cake::ActiveModel>::new()
                .add(cake::ActiveModel {
                    id: ActiveValue::set(1),
                    name: ActiveValue::set("Apple Pie".to_owned()),
                })
                .build(DbBackend::Postgres)
                .to_string(),
            r#"INSERT INTO "cake" ("id", "name") VALUES (1, 'Apple Pie')"#,
        );
    }

    #[test]
    fn insert_3() {
        assert_eq!(
            Insert::<cake::ActiveModel>::new()
                .add(cake::Model {
                    id: 1,
                    name: "Apple Pie".to_owned(),
                })
                .build(DbBackend::Postgres)
                .to_string(),
            r#"INSERT INTO "cake" ("id", "name") VALUES (1, 'Apple Pie')"#,
        );
    }

    #[test]
    fn insert_4() {
        assert_eq!(
            Insert::<cake::ActiveModel>::new()
                .add_many(vec![
                    cake::Model {
                        id: 1,
                        name: "Apple Pie".to_owned(),
                    },
                    cake::Model {
                        id: 2,
                        name: "Orange Scone".to_owned(),
                    }
                ])
                .build(DbBackend::Postgres)
                .to_string(),
            r#"INSERT INTO "cake" ("id", "name") VALUES (1, 'Apple Pie'), (2, 'Orange Scone')"#,
        );
    }

    #[test]
    #[should_panic(expected = "columns mismatch")]
    fn insert_5() {
        let apple = cake::ActiveModel {
            name: ActiveValue::set("Apple".to_owned()),
            ..Default::default()
        };
        let orange = cake::ActiveModel {
            id: ActiveValue::set(2),
            name: ActiveValue::set("Orange".to_owned()),
        };
        assert_eq!(
            Insert::<cake::ActiveModel>::new()
                .add_many(vec![apple, orange])
                .build(DbBackend::Postgres)
                .to_string(),
            r#"INSERT INTO "cake" ("id", "name") VALUES (NULL, 'Apple'), (2, 'Orange')"#,
        );
    }
}
