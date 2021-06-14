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
    use crate::tests_cfg::fruit;
    use crate::{entity::*, MockDatabase, MockRow, QueryErr};
    use futures::TryStreamExt;
    use sea_query::Value;

    #[async_std::test]
    async fn into_stream() -> Result<(), QueryErr> {
        // TODO: auto impl
        impl From<fruit::Model> for MockRow {
            fn from(model: fruit::Model) -> Self {
                let map = maplit::btreemap! {
                    "id" => Into::<Value>::into(model.id),
                    "name" => Into::<Value>::into(model.name),
                    "cake_id" => Into::<Value>::into(model.cake_id),
                };
                map.into()
            }
        }

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

        let db = MockDatabase::new()
            .append_query_results(vec![
                // TODO: take any IntoMockRow
                page1
                    .clone()
                    .into_iter()
                    .map(|model| Into::<MockRow>::into(model))
                    .collect(),
                page2
                    .clone()
                    .into_iter()
                    .map(|model| Into::<MockRow>::into(model))
                    .collect(),
            ])
            .into_database();

        let mut fruit_stream = fruit::Entity::find().paginate(&db, 2).into_stream();

        assert_eq!(fruit_stream.try_next().await?, Some(page1));

        assert_eq!(fruit_stream.try_next().await?, Some(page2));

        assert!(fruit_stream.try_next().await.is_err());
        // Should return None in live connection
        // assert_eq!(
        //     fruit_stream.try_next().await?,
        //     None
        // );

        Ok(())
    }
}
