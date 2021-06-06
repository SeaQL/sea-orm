use crate::{Connection, Database, QueryErr, SelectorTrait};
use futures::Stream;
use async_stream::stream;
use std::{marker::PhantomData, pin::Pin};
use sea_query::{Alias, Expr, SelectStatement};

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
    pub async fn fetch_page(&mut self, page: usize) -> Result<Vec<S::Item>, QueryErr> {
        self.query.limit(self.page_size as u64).offset((self.page_size * page) as u64);
        let builder = self.db.get_query_builder_backend();
        let stmt = self.query.build(builder).into();
        let rows = self.db.get_connection().query_all(stmt).await?;
        let mut buffer = Vec::with_capacity(rows.len());
        for row in rows.into_iter() {
            // TODO: Error handling
            buffer.push(S::from_raw_query_result(row).map_err(|_e| QueryErr)?);
        }
        Ok(buffer)
    }

    pub async fn fetch(&mut self) -> Result<Vec<S::Item>, QueryErr> {
        self.fetch_page(self.page).await
    }

    pub async fn num_pages(&mut self) -> Result<usize, QueryErr> {
        let builder = self.db.get_query_builder_backend();
        let stmt = SelectStatement::new()
            .expr(Expr::cust("COUNT(*) AS num_rows"))
            .from_subquery(
                self.query.clone().reset_limit().reset_offset().to_owned(),
                Alias::new("sub_query")
            )
            .build(builder)
            .into();
        let result = match self.db.get_connection().query_one(stmt).await? {
            Some(res) => res,
            None => return Ok(0),
        };
        let num_rows = result.try_get::<i32>("", "num_rows").map_err(|_e| QueryErr)? as usize;
        let num_pages = (num_rows / self.page_size) + (num_rows % self.page_size > 0) as usize;
        Ok(num_pages)
    }

    pub fn next(&mut self) {
        self.page += 1;
    }

    pub async fn fetch_and_next(&mut self) -> Result<Option<Vec<S::Item>>, QueryErr> {
        let vec = self.fetch().await?;
        self.next();
        let opt = if !vec.is_empty() {
            Some(vec)
        } else {
            None
        };
        Ok(opt)
    }

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
