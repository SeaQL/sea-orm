use crate::{Database, QueryErr, SelectorTrait};
use async_stream::stream;
use futures::Stream;
use sea_query::{Alias, Expr, SelectStatement};
use std::{marker::PhantomData, pin::Pin};

pub type PinBoxStream<'db, Item> = Pin<Box<dyn Stream<Item = Item> + 'db>>;

#[derive(Clone, Debug)]
pub struct Paginator<'db, S>
where
    S: SelectorTrait + 'db,
{
    pub(crate) query: SelectStatement,
    pub(crate) page: usize,
    pub(crate) page_size: usize,
    pub(crate) db: &'db Database,
    pub(crate) selector: PhantomData<S>,
}

impl<'db, S> Paginator<'db, S>
where
    S: SelectorTrait + 'db,
{
    /// Fetch a specific page
    pub async fn fetch_page(&self, page: usize) -> Result<Vec<S::Item>, QueryErr> {
        let query = self
            .query
            .clone()
            .limit(self.page_size as u64)
            .offset((self.page_size * page) as u64)
            .to_owned();
        let builder = self.db.get_query_builder_backend();
        let stmt = builder.build_select_statement(&query);
        let rows = self.db.get_connection().query_all(stmt).await?;
        let mut buffer = Vec::with_capacity(rows.len());
        for row in rows.into_iter() {
            // TODO: Error handling
            buffer.push(S::from_raw_query_result(row).map_err(|_e| QueryErr)?);
        }
        Ok(buffer)
    }

    /// Fetch the current page
    pub async fn fetch(&self) -> Result<Vec<S::Item>, QueryErr> {
        self.fetch_page(self.page).await
    }

    /// Get the total number of pages
    pub async fn num_pages(&self) -> Result<usize, QueryErr> {
        let builder = self.db.get_query_builder_backend();
        let stmt = builder.build_select_statement(
            SelectStatement::new()
                .expr(Expr::cust("COUNT(*) AS num_rows"))
                .from_subquery(
                    self.query.clone().reset_limit().reset_offset().to_owned(),
                    Alias::new("sub_query"),
                ),
        );
        let result = match self.db.get_connection().query_one(stmt).await? {
            Some(res) => res,
            None => return Ok(0),
        };
        let num_rows = result
            .try_get::<i32>("", "num_rows")
            .map_err(|_e| QueryErr)? as usize;
        let num_pages = (num_rows / self.page_size) + (num_rows % self.page_size > 0) as usize;
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
    pub async fn fetch_and_next(&mut self) -> Result<Option<Vec<S::Item>>, QueryErr> {
        let vec = self.fetch().await?;
        self.next();
        let opt = if !vec.is_empty() { Some(vec) } else { None };
        Ok(opt)
    }

    /// Convert self into an async stream
    pub fn into_stream(mut self) -> PinBoxStream<'db, Result<Vec<S::Item>, QueryErr>> {
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

#[cfg(test)]
#[cfg(feature = "mock")]
mod tests {
    use crate::entity::prelude::*;
    use crate::tests_cfg::{*, util::*};
    use crate::{Database, MockDatabase, QueryErr};
    use futures::TryStreamExt;
    use sea_query::{Alias, Expr, SelectStatement, Value};

    fn setup() -> (Database, Vec<Vec<fruit::Model>>) {
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

        let db = MockDatabase::new()
            .append_query_results(vec![page1.clone(), page2.clone(), page3.clone()])
            .into_database();

        (db, vec![page1, page2, page3])
    }

    fn setup_num_rows() -> (Database, i32) {
        let num_rows = 3;
        let db = MockDatabase::new()
            .append_query_results(vec![vec![maplit::btreemap! {
                "num_rows" => Into::<Value>::into(num_rows),
            }]])
            .into_database();

        (db, num_rows)
    }

    #[async_std::test]
    async fn fetch_page() -> Result<(), QueryErr> {
        let (db, pages) = setup();

        let paginator = fruit::Entity::find().paginate(&db, 2);

        assert_eq!(paginator.fetch_page(0).await?, pages[0].clone());
        assert_eq!(paginator.fetch_page(1).await?, pages[1].clone());
        assert_eq!(paginator.fetch_page(2).await?, pages[2].clone());

        let select = SelectStatement::new()
            .exprs(vec![
                Expr::tbl(fruit::Entity, fruit::Column::Id),
                Expr::tbl(fruit::Entity, fruit::Column::Name),
                Expr::tbl(fruit::Entity, fruit::Column::CakeId),
            ])
            .from(fruit::Entity)
            .to_owned();

        let stmts = vec![
            select.clone().offset(0).limit(2).to_owned(),
            select.clone().offset(2).limit(2).to_owned(),
            select.clone().offset(4).limit(2).to_owned(),
        ];
        let query_builder = db.get_query_builder_backend();
        let mut logs = get_mock_transaction_log(db);
        logs = match_transaction_log(logs, stmts, &query_builder);

        Ok(())
    }

    #[async_std::test]
    async fn fetch() -> Result<(), QueryErr> {
        let (db, pages) = setup();

        let mut paginator = fruit::Entity::find().paginate(&db, 2);

        assert_eq!(paginator.fetch().await?, pages[0].clone());
        paginator.next();

        assert_eq!(paginator.fetch().await?, pages[1].clone());
        paginator.next();

        assert_eq!(paginator.fetch().await?, pages[2].clone());

        let select = SelectStatement::new()
            .exprs(vec![
                Expr::tbl(fruit::Entity, fruit::Column::Id),
                Expr::tbl(fruit::Entity, fruit::Column::Name),
                Expr::tbl(fruit::Entity, fruit::Column::CakeId),
            ])
            .from(fruit::Entity)
            .to_owned();

        let stmts = vec![
            select.clone().offset(0).limit(2).to_owned(),
            select.clone().offset(2).limit(2).to_owned(),
            select.clone().offset(4).limit(2).to_owned(),
        ];
        let query_builder = db.get_query_builder_backend();
        let mut logs = get_mock_transaction_log(db);
        logs = match_transaction_log(logs, stmts, &query_builder);

        Ok(())
    }

    #[async_std::test]
    async fn num_pages() -> Result<(), QueryErr> {
        let (db, num_rows) = setup_num_rows();

        let num_rows = num_rows as usize;
        let page_size = 2_usize;
        let num_pages = (num_rows / page_size) + (num_rows % page_size > 0) as usize;
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
            .expr(Expr::cust("COUNT(*) AS num_rows"))
            .from_subquery(sub_query, Alias::new("sub_query"))
            .to_owned();

        let stmts = vec![select];
        let query_builder = db.get_query_builder_backend();
        let mut logs = get_mock_transaction_log(db);
        logs = match_transaction_log(logs, stmts, &query_builder);

        Ok(())
    }

    #[async_std::test]
    async fn next_and_cur_page() -> Result<(), QueryErr> {
        let (db, _) = setup();

        let mut paginator = fruit::Entity::find().paginate(&db, 2);

        assert_eq!(paginator.cur_page(), 0);
        paginator.next();

        assert_eq!(paginator.cur_page(), 1);
        paginator.next();

        assert_eq!(paginator.cur_page(), 2);

        Ok(())
    }

    #[async_std::test]
    async fn fetch_and_next() -> Result<(), QueryErr> {
        let (db, pages) = setup();

        let mut paginator = fruit::Entity::find().paginate(&db, 2);

        assert_eq!(paginator.cur_page(), 0);
        assert_eq!(paginator.fetch_and_next().await?, Some(pages[0].clone()));

        assert_eq!(paginator.cur_page(), 1);
        assert_eq!(paginator.fetch_and_next().await?, Some(pages[1].clone()));

        assert_eq!(paginator.cur_page(), 2);
        assert_eq!(paginator.fetch_and_next().await?, None);

        let select = SelectStatement::new()
            .exprs(vec![
                Expr::tbl(fruit::Entity, fruit::Column::Id),
                Expr::tbl(fruit::Entity, fruit::Column::Name),
                Expr::tbl(fruit::Entity, fruit::Column::CakeId),
            ])
            .from(fruit::Entity)
            .to_owned();

        let stmts = vec![
            select.clone().offset(0).limit(2).to_owned(),
            select.clone().offset(2).limit(2).to_owned(),
            select.clone().offset(4).limit(2).to_owned(),
        ];
        let query_builder = db.get_query_builder_backend();
        let mut logs = get_mock_transaction_log(db);
        logs = match_transaction_log(logs, stmts, &query_builder);

        Ok(())
    }

    #[async_std::test]
    async fn into_stream() -> Result<(), QueryErr> {
        let (db, pages) = setup();

        let mut fruit_stream = fruit::Entity::find().paginate(&db, 2).into_stream();

        assert_eq!(fruit_stream.try_next().await?, Some(pages[0].clone()));
        assert_eq!(fruit_stream.try_next().await?, Some(pages[1].clone()));
        assert_eq!(fruit_stream.try_next().await?, None);

        drop(fruit_stream);

        let select = SelectStatement::new()
            .exprs(vec![
                Expr::tbl(fruit::Entity, fruit::Column::Id),
                Expr::tbl(fruit::Entity, fruit::Column::Name),
                Expr::tbl(fruit::Entity, fruit::Column::CakeId),
            ])
            .from(fruit::Entity)
            .to_owned();

        let stmts = vec![
            select.clone().offset(0).limit(2).to_owned(),
            select.clone().offset(2).limit(2).to_owned(),
            select.clone().offset(4).limit(2).to_owned(),
        ];
        let query_builder = db.get_query_builder_backend();
        let mut logs = get_mock_transaction_log(db);
        logs = match_transaction_log(logs, stmts[0..1].to_vec(), &query_builder);
        logs = match_transaction_log(logs, stmts[1..].to_vec(), &query_builder);

        Ok(())
    }
}
