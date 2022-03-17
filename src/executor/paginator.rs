use crate::{
    error::*, ConnectionTrait, DbBackend, EntityTrait, FromQueryResult, Select, SelectModel,
    SelectTwo, SelectTwoModel, Selector, SelectorRaw, SelectorTrait,
};
use async_stream::stream;
use futures::Stream;
use sea_query::{Alias, Expr, SelectStatement};
use std::{marker::PhantomData, pin::Pin};

/// Pin a Model so that stream operations can be performed on the model
pub type PinBoxStream<'db, Item> = Pin<Box<dyn Stream<Item = Item> + 'db>>;

/// Defined a structure to handle pagination of a result from a query operation on a Model
#[derive(Clone, Debug)]
pub struct Paginator<'db, C, S>
where
    C: ConnectionTrait,
    S: SelectorTrait + 'db,
{
    pub(crate) query: SelectStatement,
    pub(crate) page: usize,
    pub(crate) page_size: usize,
    pub(crate) db: &'db C,
    pub(crate) selector: PhantomData<S>,
}

// LINT: warn if paginator is used without an order by clause

impl<'db, C, S> Paginator<'db, C, S>
where
    C: ConnectionTrait,
    S: SelectorTrait + 'db,
{
    /// Fetch a specific page; page index starts from zero
    pub async fn fetch_page(&self, page: usize) -> Result<Vec<S::Item>, DbErr> {
        let query = self
            .query
            .clone()
            .limit(self.page_size as u64)
            .offset((self.page_size * page) as u64)
            .to_owned();
        let builder = self.db.get_database_backend();
        let stmt = builder.build(&query);
        let rows = self.db.query_all(stmt).await?;
        let mut buffer = Vec::with_capacity(rows.len());
        for row in rows.into_iter() {
            // TODO: Error handling
            buffer.push(S::from_raw_query_result(row)?);
        }
        Ok(buffer)
    }

    /// Fetch the current page
    pub async fn fetch(&self) -> Result<Vec<S::Item>, DbErr> {
        self.fetch_page(self.page).await
    }

    /// Get the total number of items
    pub async fn num_items(&self) -> Result<usize, DbErr> {
        let builder = self.db.get_database_backend();
        let stmt = builder.build(
            SelectStatement::new()
                .expr(Expr::cust("COUNT(*) AS num_items"))
                .from_subquery(
                    self.query.clone().reset_limit().reset_offset().to_owned(),
                    Alias::new("sub_query"),
                ),
        );
        let result = match self.db.query_one(stmt).await? {
            Some(res) => res,
            None => return Ok(0),
        };
        let num_items = match builder {
            DbBackend::Postgres => result.try_get::<i64>("", "num_items")? as usize,
            _ => result.try_get::<i32>("", "num_items")? as usize,
        };
        Ok(num_items)
    }

    /// Get the total number of pages
    pub async fn num_pages(&self) -> Result<usize, DbErr> {
        let num_items = self.num_items().await?;
        let num_pages = (num_items / self.page_size) + (num_items % self.page_size > 0) as usize;
        Ok(num_pages)
    }

    /// Increment the page counter
    pub fn next(&mut self) {
        self.page += 1;
    }

    /// Get current page number
    pub fn cur_page(&self) -> usize {
        self.page
    }

    /// Fetch one page and increment the page counter
    ///
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let owned_db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_query_results(vec![
    /// #         vec![cake::Model {
    /// #             id: 1,
    /// #             name: "Cake".to_owned(),
    /// #         }],
    /// #         vec![],
    /// #     ])
    /// #     .into_connection();
    /// # let db = &owned_db;
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake};
    /// let mut cake_pages = cake::Entity::find()
    ///     .order_by_asc(cake::Column::Id)
    ///     .paginate(db, 50);
    ///
    /// while let Some(cakes) = cake_pages.fetch_and_next().await? {
    ///     // Do something on cakes: Vec<cake::Model>
    /// }
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch_and_next(&mut self) -> Result<Option<Vec<S::Item>>, DbErr> {
        let vec = self.fetch().await?;
        self.next();
        let opt = if !vec.is_empty() { Some(vec) } else { None };
        Ok(opt)
    }

    /// Convert self into an async stream
    ///
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let owned_db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_query_results(vec![
    /// #         vec![cake::Model {
    /// #             id: 1,
    /// #             name: "Cake".to_owned(),
    /// #         }],
    /// #         vec![],
    /// #     ])
    /// #     .into_connection();
    /// # let db = &owned_db;
    /// #
    /// use futures::TryStreamExt;
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake};
    /// let mut cake_stream = cake::Entity::find()
    ///     .order_by_asc(cake::Column::Id)
    ///     .paginate(db, 50)
    ///     .into_stream();
    ///
    /// while let Some(cakes) = cake_stream.try_next().await? {
    ///     // Do something on cakes: Vec<cake::Model>
    /// }
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_stream(mut self) -> PinBoxStream<'db, Result<Vec<S::Item>, DbErr>> {
        Box::pin(stream! {
            loop {
                if let Some(vec) = self.fetch_and_next().await? {
                    yield Ok(vec);
                } else {
                    break
                }
            }
        })
    }
}

#[async_trait::async_trait]
/// A Trait for any type that can paginate results
pub trait PaginatorTrait<'db, C>
where
    C: ConnectionTrait,
{
    /// Select operation
    type Selector: SelectorTrait + Send + Sync + 'db;

    /// Paginate the result of a select operation.
    fn paginate(self, db: &'db C, page_size: usize) -> Paginator<'db, C, Self::Selector>;

    /// Perform a count on the paginated results
    async fn count(self, db: &'db C) -> Result<usize, DbErr>
    where
        Self: Send + Sized,
    {
        self.paginate(db, 1).num_items().await
    }
}

impl<'db, C, S> PaginatorTrait<'db, C> for Selector<S>
where
    C: ConnectionTrait,
    S: SelectorTrait + Send + Sync + 'db,
{
    type Selector = S;

    fn paginate(self, db: &'db C, page_size: usize) -> Paginator<'db, C, S> {
        Paginator {
            query: self.query,
            page: 0,
            page_size,
            db,
            selector: PhantomData,
        }
    }
}

impl<'db, C, S> PaginatorTrait<'db, C> for SelectorRaw<S>
where
    C: ConnectionTrait,
    S: SelectorTrait + Send + Sync + 'db,
{
    type Selector = S;
    fn paginate(self, db: &'db C, page_size: usize) -> Paginator<'db, C, S> {
        let sql = &self.stmt.sql[6..];
        let mut query = SelectStatement::new();
        query.expr(if let Some(values) = self.stmt.values {
            Expr::cust_with_values(sql, values.0)
        } else {
            Expr::cust(sql)
        });

        Paginator {
            query,
            page: 0,
            page_size,
            db,
            selector: PhantomData,
        }
    }
}

impl<'db, C, M, E> PaginatorTrait<'db, C> for Select<E>
where
    C: ConnectionTrait,
    E: EntityTrait<Model = M>,
    M: FromQueryResult + Sized + Send + Sync + 'db,
{
    type Selector = SelectModel<M>;

    fn paginate(self, db: &'db C, page_size: usize) -> Paginator<'db, C, Self::Selector> {
        self.into_model().paginate(db, page_size)
    }
}

impl<'db, C, M, N, E, F> PaginatorTrait<'db, C> for SelectTwo<E, F>
where
    C: ConnectionTrait,
    E: EntityTrait<Model = M>,
    F: EntityTrait<Model = N>,
    M: FromQueryResult + Sized + Send + Sync + 'db,
    N: FromQueryResult + Sized + Send + Sync + 'db,
{
    type Selector = SelectTwoModel<M, N>;

    fn paginate(self, db: &'db C, page_size: usize) -> Paginator<'db, C, Self::Selector> {
        self.into_model().paginate(db, page_size)
    }
}

#[cfg(test)]
#[cfg(feature = "mock")]
mod tests {
    use super::*;
    use crate::entity::prelude::*;
    use crate::{tests_cfg::*, ConnectionTrait, Statement};
    use crate::{DatabaseConnection, DbBackend, MockDatabase, Transaction};
    use futures::TryStreamExt;
    use once_cell::sync::Lazy;
    use sea_query::{Alias, Expr, SelectStatement, Value};

    static RAW_STMT: Lazy<Statement> = Lazy::new(|| {
        Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT "fruit"."id", "fruit"."name", "fruit"."cake_id" FROM "fruit""#,
            vec![],
        )
    });

    fn setup() -> (DatabaseConnection, Vec<Vec<fruit::Model>>) {
        let page1 = vec![
            fruit::Model {
                id: 1,
                name: "Blueberry".into(),
                cake_id: Some(1),
            },
            fruit::Model {
                id: 2,
                name: "Rasberry".into(),
                cake_id: Some(1),
            },
        ];

        let page2 = vec![fruit::Model {
            id: 3,
            name: "Strawberry".into(),
            cake_id: Some(2),
        }];

        let page3 = Vec::<fruit::Model>::new();

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results(vec![page1.clone(), page2.clone(), page3.clone()])
            .into_connection();

        (db, vec![page1, page2, page3])
    }

    fn setup_num_items() -> (DatabaseConnection, i64) {
        let num_items = 3;
        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results(vec![vec![maplit::btreemap! {
                "num_items" => Into::<Value>::into(num_items),
            }]])
            .into_connection();

        (db, num_items)
    }

    #[smol_potat::test]
    async fn fetch_page() -> Result<(), DbErr> {
        let (db, pages) = setup();

        let paginator = fruit::Entity::find().paginate(&db, 2);

        assert_eq!(paginator.fetch_page(0).await?, pages[0].clone());
        assert_eq!(paginator.fetch_page(1).await?, pages[1].clone());
        assert_eq!(paginator.fetch_page(2).await?, pages[2].clone());

        let mut select = SelectStatement::new()
            .exprs(vec![
                Expr::tbl(fruit::Entity, fruit::Column::Id),
                Expr::tbl(fruit::Entity, fruit::Column::Name),
                Expr::tbl(fruit::Entity, fruit::Column::CakeId),
            ])
            .from(fruit::Entity)
            .to_owned();

        let query_builder = db.get_database_backend();
        let stmts = vec![
            query_builder.build(select.clone().offset(0).limit(2)),
            query_builder.build(select.clone().offset(2).limit(2)),
            query_builder.build(select.offset(4).limit(2)),
        ];

        assert_eq!(db.into_transaction_log(), Transaction::wrap(stmts));
        Ok(())
    }

    #[smol_potat::test]
    async fn fetch_page_raw() -> Result<(), DbErr> {
        let (db, pages) = setup();

        let paginator = fruit::Entity::find()
            .from_raw_sql(RAW_STMT.clone())
            .paginate(&db, 2);

        assert_eq!(paginator.fetch_page(0).await?, pages[0].clone());
        assert_eq!(paginator.fetch_page(1).await?, pages[1].clone());
        assert_eq!(paginator.fetch_page(2).await?, pages[2].clone());

        let mut select = SelectStatement::new()
            .exprs(vec![
                Expr::tbl(fruit::Entity, fruit::Column::Id),
                Expr::tbl(fruit::Entity, fruit::Column::Name),
                Expr::tbl(fruit::Entity, fruit::Column::CakeId),
            ])
            .from(fruit::Entity)
            .to_owned();

        let query_builder = db.get_database_backend();
        let stmts = vec![
            query_builder.build(select.clone().offset(0).limit(2)),
            query_builder.build(select.clone().offset(2).limit(2)),
            query_builder.build(select.offset(4).limit(2)),
        ];

        assert_eq!(db.into_transaction_log(), Transaction::wrap(stmts));
        Ok(())
    }

    #[smol_potat::test]
    async fn fetch() -> Result<(), DbErr> {
        let (db, pages) = setup();

        let mut paginator = fruit::Entity::find().paginate(&db, 2);

        assert_eq!(paginator.fetch().await?, pages[0].clone());
        paginator.next();

        assert_eq!(paginator.fetch().await?, pages[1].clone());
        paginator.next();

        assert_eq!(paginator.fetch().await?, pages[2].clone());

        let mut select = SelectStatement::new()
            .exprs(vec![
                Expr::tbl(fruit::Entity, fruit::Column::Id),
                Expr::tbl(fruit::Entity, fruit::Column::Name),
                Expr::tbl(fruit::Entity, fruit::Column::CakeId),
            ])
            .from(fruit::Entity)
            .to_owned();

        let query_builder = db.get_database_backend();
        let stmts = vec![
            query_builder.build(select.clone().offset(0).limit(2)),
            query_builder.build(select.clone().offset(2).limit(2)),
            query_builder.build(select.offset(4).limit(2)),
        ];

        assert_eq!(db.into_transaction_log(), Transaction::wrap(stmts));
        Ok(())
    }

    #[smol_potat::test]
    async fn fetch_raw() -> Result<(), DbErr> {
        let (db, pages) = setup();

        let mut paginator = fruit::Entity::find()
            .from_raw_sql(RAW_STMT.clone())
            .paginate(&db, 2);

        assert_eq!(paginator.fetch().await?, pages[0].clone());
        paginator.next();

        assert_eq!(paginator.fetch().await?, pages[1].clone());
        paginator.next();

        assert_eq!(paginator.fetch().await?, pages[2].clone());

        let mut select = SelectStatement::new()
            .exprs(vec![
                Expr::tbl(fruit::Entity, fruit::Column::Id),
                Expr::tbl(fruit::Entity, fruit::Column::Name),
                Expr::tbl(fruit::Entity, fruit::Column::CakeId),
            ])
            .from(fruit::Entity)
            .to_owned();

        let query_builder = db.get_database_backend();
        let stmts = vec![
            query_builder.build(select.clone().offset(0).limit(2)),
            query_builder.build(select.clone().offset(2).limit(2)),
            query_builder.build(select.offset(4).limit(2)),
        ];

        assert_eq!(db.into_transaction_log(), Transaction::wrap(stmts));
        Ok(())
    }

    #[smol_potat::test]
    async fn num_pages() -> Result<(), DbErr> {
        let (db, num_items) = setup_num_items();

        let num_items = num_items as usize;
        let page_size = 2_usize;
        let num_pages = (num_items / page_size) + (num_items % page_size > 0) as usize;
        let paginator = fruit::Entity::find().paginate(&db, page_size);

        assert_eq!(paginator.num_pages().await?, num_pages);

        let sub_query = SelectStatement::new()
            .exprs(vec![
                Expr::tbl(fruit::Entity, fruit::Column::Id),
                Expr::tbl(fruit::Entity, fruit::Column::Name),
                Expr::tbl(fruit::Entity, fruit::Column::CakeId),
            ])
            .from(fruit::Entity)
            .to_owned();

        let select = SelectStatement::new()
            .expr(Expr::cust("COUNT(*) AS num_items"))
            .from_subquery(sub_query, Alias::new("sub_query"))
            .to_owned();

        let query_builder = db.get_database_backend();
        let stmts = vec![query_builder.build(&select)];

        assert_eq!(db.into_transaction_log(), Transaction::wrap(stmts));
        Ok(())
    }

    #[smol_potat::test]
    async fn num_pages_raw() -> Result<(), DbErr> {
        let (db, num_items) = setup_num_items();

        let num_items = num_items as usize;
        let page_size = 2_usize;
        let num_pages = (num_items / page_size) + (num_items % page_size > 0) as usize;
        let paginator = fruit::Entity::find()
            .from_raw_sql(RAW_STMT.clone())
            .paginate(&db, page_size);

        assert_eq!(paginator.num_pages().await?, num_pages);

        let sub_query = SelectStatement::new()
            .exprs(vec![
                Expr::tbl(fruit::Entity, fruit::Column::Id),
                Expr::tbl(fruit::Entity, fruit::Column::Name),
                Expr::tbl(fruit::Entity, fruit::Column::CakeId),
            ])
            .from(fruit::Entity)
            .to_owned();

        let select = SelectStatement::new()
            .expr(Expr::cust("COUNT(*) AS num_items"))
            .from_subquery(sub_query, Alias::new("sub_query"))
            .to_owned();

        let query_builder = db.get_database_backend();
        let stmts = vec![query_builder.build(&select)];

        assert_eq!(db.into_transaction_log(), Transaction::wrap(stmts));
        Ok(())
    }

    #[smol_potat::test]
    async fn next_and_cur_page() -> Result<(), DbErr> {
        let (db, _) = setup();

        let mut paginator = fruit::Entity::find().paginate(&db, 2);

        assert_eq!(paginator.cur_page(), 0);
        paginator.next();

        assert_eq!(paginator.cur_page(), 1);
        paginator.next();

        assert_eq!(paginator.cur_page(), 2);
        Ok(())
    }

    #[smol_potat::test]
    async fn next_and_cur_page_raw() -> Result<(), DbErr> {
        let (db, _) = setup();

        let mut paginator = fruit::Entity::find()
            .from_raw_sql(RAW_STMT.clone())
            .paginate(&db, 2);

        assert_eq!(paginator.cur_page(), 0);
        paginator.next();

        assert_eq!(paginator.cur_page(), 1);
        paginator.next();

        assert_eq!(paginator.cur_page(), 2);
        Ok(())
    }

    #[smol_potat::test]
    async fn fetch_and_next() -> Result<(), DbErr> {
        let (db, pages) = setup();

        let mut paginator = fruit::Entity::find().paginate(&db, 2);

        assert_eq!(paginator.cur_page(), 0);
        assert_eq!(paginator.fetch_and_next().await?, Some(pages[0].clone()));

        assert_eq!(paginator.cur_page(), 1);
        assert_eq!(paginator.fetch_and_next().await?, Some(pages[1].clone()));

        assert_eq!(paginator.cur_page(), 2);
        assert_eq!(paginator.fetch_and_next().await?, None);

        let mut select = SelectStatement::new()
            .exprs(vec![
                Expr::tbl(fruit::Entity, fruit::Column::Id),
                Expr::tbl(fruit::Entity, fruit::Column::Name),
                Expr::tbl(fruit::Entity, fruit::Column::CakeId),
            ])
            .from(fruit::Entity)
            .to_owned();

        let query_builder = db.get_database_backend();
        let stmts = vec![
            query_builder.build(select.clone().offset(0).limit(2)),
            query_builder.build(select.clone().offset(2).limit(2)),
            query_builder.build(select.offset(4).limit(2)),
        ];

        assert_eq!(db.into_transaction_log(), Transaction::wrap(stmts));
        Ok(())
    }

    #[smol_potat::test]
    async fn fetch_and_next_raw() -> Result<(), DbErr> {
        let (db, pages) = setup();

        let mut paginator = fruit::Entity::find()
            .from_raw_sql(RAW_STMT.clone())
            .paginate(&db, 2);

        assert_eq!(paginator.cur_page(), 0);
        assert_eq!(paginator.fetch_and_next().await?, Some(pages[0].clone()));

        assert_eq!(paginator.cur_page(), 1);
        assert_eq!(paginator.fetch_and_next().await?, Some(pages[1].clone()));

        assert_eq!(paginator.cur_page(), 2);
        assert_eq!(paginator.fetch_and_next().await?, None);

        let mut select = SelectStatement::new()
            .exprs(vec![
                Expr::tbl(fruit::Entity, fruit::Column::Id),
                Expr::tbl(fruit::Entity, fruit::Column::Name),
                Expr::tbl(fruit::Entity, fruit::Column::CakeId),
            ])
            .from(fruit::Entity)
            .to_owned();

        let query_builder = db.get_database_backend();
        let stmts = vec![
            query_builder.build(select.clone().offset(0).limit(2)),
            query_builder.build(select.clone().offset(2).limit(2)),
            query_builder.build(select.offset(4).limit(2)),
        ];

        assert_eq!(db.into_transaction_log(), Transaction::wrap(stmts));
        Ok(())
    }

    #[smol_potat::test]
    async fn into_stream() -> Result<(), DbErr> {
        let (db, pages) = setup();

        let mut fruit_stream = fruit::Entity::find().paginate(&db, 2).into_stream();

        assert_eq!(fruit_stream.try_next().await?, Some(pages[0].clone()));
        assert_eq!(fruit_stream.try_next().await?, Some(pages[1].clone()));
        assert_eq!(fruit_stream.try_next().await?, None);

        drop(fruit_stream);

        let mut select = SelectStatement::new()
            .exprs(vec![
                Expr::tbl(fruit::Entity, fruit::Column::Id),
                Expr::tbl(fruit::Entity, fruit::Column::Name),
                Expr::tbl(fruit::Entity, fruit::Column::CakeId),
            ])
            .from(fruit::Entity)
            .to_owned();

        let query_builder = db.get_database_backend();
        let stmts = vec![
            query_builder.build(select.clone().offset(0).limit(2)),
            query_builder.build(select.clone().offset(2).limit(2)),
            query_builder.build(select.offset(4).limit(2)),
        ];

        assert_eq!(db.into_transaction_log(), Transaction::wrap(stmts));
        Ok(())
    }

    #[smol_potat::test]
    async fn into_stream_raw() -> Result<(), DbErr> {
        let (db, pages) = setup();

        let mut fruit_stream = fruit::Entity::find()
            .from_raw_sql(RAW_STMT.clone())
            .paginate(&db, 2)
            .into_stream();

        assert_eq!(fruit_stream.try_next().await?, Some(pages[0].clone()));
        assert_eq!(fruit_stream.try_next().await?, Some(pages[1].clone()));
        assert_eq!(fruit_stream.try_next().await?, None);

        drop(fruit_stream);

        let mut select = SelectStatement::new()
            .exprs(vec![
                Expr::tbl(fruit::Entity, fruit::Column::Id),
                Expr::tbl(fruit::Entity, fruit::Column::Name),
                Expr::tbl(fruit::Entity, fruit::Column::CakeId),
            ])
            .from(fruit::Entity)
            .to_owned();

        let query_builder = db.get_database_backend();
        let stmts = vec![
            query_builder.build(select.clone().offset(0).limit(2)),
            query_builder.build(select.clone().offset(2).limit(2)),
            query_builder.build(select.offset(4).limit(2)),
        ];

        assert_eq!(db.into_transaction_log(), Transaction::wrap(stmts));
        Ok(())
    }
}
