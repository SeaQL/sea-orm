use crate::{
    ConnectionTrait, DbBackend, DbErr, EntityTrait, FromQueryResult, QueryResult, Select,
    Statement, dynamic,
};
use sea_query::{DynIden, Expr, IntoIden, SelectStatement};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct SelectModelAndDynModel<M>
where
    M: FromQueryResult,
{
    model: PhantomData<M>,
    dyn_model: dynamic::ModelType,
}

#[derive(Clone, Debug)]
pub struct DynSelector<S>
where
    S: DynSelectorTrait,
{
    pub(crate) query: SelectStatement,
    selector: S,
}

pub trait DynSelectorTrait {
    type Item: Sized;

    #[allow(clippy::wrong_self_convention)]
    fn from_raw_query_result(&self, res: QueryResult) -> Result<Self::Item, DbErr>;
}

impl<M> DynSelectorTrait for SelectModelAndDynModel<M>
where
    M: FromQueryResult + Sized,
{
    type Item = (M, dynamic::Model);

    fn from_raw_query_result(&self, res: QueryResult) -> Result<Self::Item, DbErr> {
        Ok((
            M::from_query_result(&res, "")?,
            self.dyn_model.from_query_result(&res, "")?,
        ))
    }
}

impl<E> Select<E>
where
    E: EntityTrait,
{
    pub fn select_also_dyn_model(
        mut self,
        table: DynIden,
        dyn_model: dynamic::ModelType,
    ) -> DynSelector<SelectModelAndDynModel<E::Model>> {
        for field in dyn_model.fields.iter() {
            self.query.expr(Expr::col((
                table.clone(),
                field.field().to_owned().into_iden(),
            )));
        }
        DynSelector {
            query: self.query,
            selector: SelectModelAndDynModel {
                model: PhantomData,
                dyn_model,
            },
        }
    }
}

impl<S> DynSelector<S>
where
    S: DynSelectorTrait,
{
    /// Get the SQL statement
    pub fn into_statement(self, builder: DbBackend) -> Statement {
        builder.build(&self.query)
    }

    /// Get an item from the Select query
    pub async fn one<C>(mut self, db: &C) -> Result<Option<S::Item>, DbErr>
    where
        C: ConnectionTrait,
    {
        self.query.limit(1);
        let row = db.query_one(&self.query).await?;
        match row {
            Some(row) => Ok(Some(self.selector.from_raw_query_result(row)?)),
            None => Ok(None),
        }
    }

    /// Get all items from the Select query
    pub async fn all<C>(self, db: &C) -> Result<Vec<S::Item>, DbErr>
    where
        C: ConnectionTrait,
    {
        let rows = db.query_all(&self.query).await?;
        let mut models = Vec::new();
        for row in rows.into_iter() {
            models.push(self.selector.from_raw_query_result(row)?);
        }
        Ok(models)
    }
}
