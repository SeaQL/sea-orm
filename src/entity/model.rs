use crate::{
    ActiveModelBehavior, ActiveModelTrait, ColumnTrait, ConnectionTrait, DbErr, DeleteResult,
    EntityTrait, IntoActiveModel, Iterable, Linked, PrimaryKeyArity, PrimaryKeyToColumn,
    PrimaryKeyTrait, QueryFilter, QueryResult, Related, Select, SelectModel, SelectorRaw,
    Statement, TryGetError, find_linked_recursive,
};
pub use sea_query::Value;
use sea_query::{ArrayType, ValueTuple};
use std::fmt::Debug;

/// The interface implemented by every Model — an instance of an
/// [`EntityTrait`], roughly an OOP "object" whose fields are the table's
/// columns.
///
/// Implemented automatically by `#[derive(DeriveEntityModel)]` /
/// `#[derive(DeriveModel)]`. Pairs with an [`ActiveModelTrait`] type for
/// mutations.
#[async_trait::async_trait]
pub trait ModelTrait: Clone + Send + Debug {
    /// The [`EntityTrait`] this model belongs to.
    type Entity: EntityTrait;

    /// Read the value of one column.
    fn get(&self, c: <Self::Entity as EntityTrait>::Column) -> Value;

    /// Type of the value stored by a column, used by reflection helpers
    /// such as Arrow conversion.
    fn get_value_type(c: <Self::Entity as EntityTrait>::Column) -> ArrayType;

    /// Write a value to one column. Panics if the value's type doesn't match
    /// the column; prefer [`try_set`](Self::try_set) when the value comes
    /// from untrusted input.
    fn set(&mut self, c: <Self::Entity as EntityTrait>::Column, v: Value) {
        self.try_set(c, v)
            .unwrap_or_else(|e| panic!("Failed to set value for {:?}: {e:?}", c.as_column_ref()))
    }

    /// Write a value to one column, returning an error if the value's type
    /// does not match the column.
    fn try_set(&mut self, c: <Self::Entity as EntityTrait>::Column, v: Value) -> Result<(), DbErr>;

    /// Build a [`Select`] for models related to `self` via the
    /// `Self::Entity: Related<R>` relation. Use it together with `.one(db)` /
    /// `.all(db)` to fetch the related rows.
    fn find_related<R>(&self, _: R) -> Select<R>
    where
        R: EntityTrait,
        Self::Entity: Related<R>,
    {
        <Self::Entity as Related<R>>::find_related().belongs_to(self)
    }

    /// Build a [`Select`] that follows a multi-hop link out of `self`. The
    /// hops are described by a [`Linked`] implementation.
    fn find_linked<L>(&self, l: L) -> Select<L::ToEntity>
    where
        L: Linked<FromEntity = Self::Entity>,
    {
        let tbl_alias = &format!("r{}", l.link().len() - 1);
        l.find_linked().belongs_to_tbl_alias(self, tbl_alias)
    }

    #[doc(hidden)]
    /// Find linked Models with a recursive CTE for self-referencing relation chains
    fn find_linked_recursive<L>(&self, l: L) -> Select<L::ToEntity>
    where
        L: Linked<FromEntity = Self::Entity, ToEntity = Self::Entity>,
    {
        // Have to do this because L is not Clone
        let link = l.link();
        let initial_query = self.find_linked(l);
        find_linked_recursive(initial_query, link)
    }

    /// Delete a model
    async fn delete<'a, A, C>(self, db: &'a C) -> Result<DeleteResult, DbErr>
    where
        Self: IntoActiveModel<A>,
        C: ConnectionTrait,
        A: ActiveModelTrait<Entity = Self::Entity> + ActiveModelBehavior + Send + 'a,
    {
        self.into_active_model().delete(db).await
    }

    /// Get the primary key value of the Model
    fn get_primary_key_value(&self) -> ValueTuple {
        let mut cols = <Self::Entity as EntityTrait>::PrimaryKey::iter();
        macro_rules! next {
            () => {
                self.get(cols.next().expect("Already checked arity").into_column())
            };
        }
        match <<<Self::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType as PrimaryKeyArity>::ARITY {
            1 => {
                let s1 = next!();
                ValueTuple::One(s1)
            }
            2 => {
                let s1 = next!();
                let s2 = next!();
                ValueTuple::Two(s1, s2)
            }
            3 => {
                let s1 = next!();
                let s2 = next!();
                let s3 = next!();
                ValueTuple::Three(s1, s2, s3)
            }
            len => {
                let mut vec = Vec::with_capacity(len);
                for _ in 0..len {
                    let s = next!();
                    vec.push(s);
                }
                ValueTuple::Many(vec)
            }
        }
    }
}

/// Construct a value from a [`QueryResult`] row.
///
/// Implemented for every Model via `#[derive(DeriveModel)]`, and can be
/// derived on any custom struct with `#[derive(FromQueryResult)]` to read
/// an arbitrary shape out of a `SELECT`. See
/// [`find_by_statement`](Self::find_by_statement) for executing a raw SQL
/// query that materialises into the type.
pub trait FromQueryResult: Sized {
    /// Instantiate a Model from a [QueryResult]
    ///
    /// NOTE: Please also override `from_query_result_nullable` when manually implementing.
    ///       The future default implementation will be along the lines of:
    ///
    /// ```rust,ignore
    /// fn from_query_result(res: &QueryResult, pre: &str) -> Result<Self, DbErr> {
    ///     (Self::from_query_result_nullable(res, pre)?)
    /// }
    /// ```
    fn from_query_result(res: &QueryResult, pre: &str) -> Result<Self, DbErr>;

    /// Transform the error from instantiating a Model from a [QueryResult]
    /// and converting it to an [Option]
    fn from_query_result_optional(res: &QueryResult, pre: &str) -> Result<Option<Self>, DbErr> {
        Ok(Self::from_query_result(res, pre).ok())

        // would really like to do the following, but can't without version bump:
        // match Self::from_query_result_nullable(res, pre) {
        //     Ok(v) => Ok(Some(v)),
        //     Err(TryGetError::Null(_)) => Ok(None),
        //     Err(TryGetError::DbErr(err)) => Err(err),
        // }
    }

    /// Transform the error from instantiating a Model from a [QueryResult]
    /// and converting it to an [Option]
    ///
    /// NOTE: This will most likely stop being a provided method in the next major version!
    fn from_query_result_nullable(res: &QueryResult, pre: &str) -> Result<Self, TryGetError> {
        Self::from_query_result(res, pre).map_err(TryGetError::DbErr)
    }

    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_query_results([[
    /// #         maplit::btreemap! {
    /// #             "name" => Into::<Value>::into("Chocolate Forest"),
    /// #             "num_of_cakes" => Into::<Value>::into(2),
    /// #         },
    /// #     ]])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{FromQueryResult, query::*};
    ///
    /// #[derive(Debug, PartialEq, FromQueryResult)]
    /// struct SelectResult {
    ///     name: String,
    ///     num_of_cakes: i32,
    /// }
    ///
    /// let res: Vec<SelectResult> = SelectResult::find_by_statement(Statement::from_sql_and_values(
    ///     DbBackend::Postgres,
    ///     r#"SELECT "name", COUNT(*) AS "num_of_cakes" FROM "cake" GROUP BY("name")"#,
    ///     [],
    /// ))
    /// .all(&db)
    /// .await?;
    ///
    /// assert_eq!(
    ///     res,
    ///     [SelectResult {
    ///         name: "Chocolate Forest".to_owned(),
    ///         num_of_cakes: 2,
    ///     },]
    /// );
    /// #
    /// # assert_eq!(
    /// #     db.into_transaction_log(),
    /// #     [Transaction::from_sql_and_values(
    /// #         DbBackend::Postgres,
    /// #         r#"SELECT "name", COUNT(*) AS "num_of_cakes" FROM "cake" GROUP BY("name")"#,
    /// #         []
    /// #     ),]
    /// # );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    fn find_by_statement(stmt: Statement) -> SelectorRaw<SelectModel<Self>> {
        SelectorRaw::<SelectModel<Self>>::from_statement(stmt)
    }
}

impl<T: FromQueryResult> FromQueryResult for Option<T> {
    fn from_query_result(res: &QueryResult, pre: &str) -> Result<Self, DbErr> {
        Ok(Self::from_query_result_nullable(res, pre)?)
    }

    fn from_query_result_optional(res: &QueryResult, pre: &str) -> Result<Option<Self>, DbErr> {
        match Self::from_query_result_nullable(res, pre) {
            Ok(v) => Ok(Some(v)),
            Err(TryGetError::Null(_)) => Ok(None),
            Err(TryGetError::DbErr(err)) => Err(err),
        }
    }

    fn from_query_result_nullable(res: &QueryResult, pre: &str) -> Result<Self, TryGetError> {
        match T::from_query_result_nullable(res, pre) {
            Ok(v) => Ok(Some(v)),
            Err(TryGetError::Null(_)) => Ok(None),
            Err(err @ TryGetError::DbErr(_)) => Err(err),
        }
    }
}

/// Fallible conversion into a [`ModelTrait`] value.
///
/// Implemented for [`ActiveModelTrait`] so a partially-filled `ActiveModel`
/// can be turned into a full `Model` once every column is `Set` or
/// `Unchanged`; returns [`DbErr::AttrNotSet`](crate::DbErr::AttrNotSet)
/// otherwise.
pub trait TryIntoModel<M>
where
    M: ModelTrait,
{
    /// Attempt the conversion, returning an error if a required column is
    /// not set.
    fn try_into_model(self) -> Result<M, DbErr>;
}

impl<M> TryIntoModel<M> for M
where
    M: ModelTrait,
{
    fn try_into_model(self) -> Result<M, DbErr> {
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        IntoActiveModel, Unchanged,
        prelude::*,
        tests_cfg::{cake, filling, fruit, post},
    };

    #[test]
    fn test_model() {
        fn filter_by_column(col: post::Column) -> Expr {
            col.eq("attribute")
        }

        fn get_value_from(model: &post::Model, col: post::Column) {
            let value: i32 = model.get(col).unwrap();
            assert_eq!(value, 12);
        }

        let model = post::Model {
            id: 12,
            user_id: 14,
            title: "hello".into(),
        };

        get_value_from(&model, post::COLUMN.id.0);
        filter_by_column(post::COLUMN.title.0);

        let filling = filling::Model {
            id: 12,
            name: "".into(),
            vendor_id: None,
            ignored_attr: 24,
        };

        let filling_am = filling::ActiveModel {
            id: Unchanged(12),
            name: Unchanged("".into()),
            vendor_id: Unchanged(None),
        };

        assert_eq!(filling.into_active_model(), filling_am);

        let filling_ex = filling::ActiveModelEx {
            id: Unchanged(12),
            name: Unchanged("".into()),
            vendor_id: Unchanged(None),
            ingredients: ActiveHasMany::NotSet,
        };

        assert_eq!(filling_am.into_ex(), filling_ex);

        let cake_ex = cake::ModelEx {
            id: 12,
            name: "C".into(),
            fruit: HasOne::loaded(fruit::Model {
                id: 13,
                name: "F".into(),
                cake_id: Some(12),
            }),
            fillings: HasMany::Loaded(vec![
                filling::Model {
                    id: 14,
                    name: "FF".into(),
                    vendor_id: None,
                    ignored_attr: 1,
                }
                .into(),
            ]),
        };

        let cake_am = cake::ActiveModelEx {
            id: Unchanged(12),
            name: Unchanged("C".into()),
            fruit: ActiveHasOne::Set(
                fruit::ActiveModelEx {
                    id: Unchanged(13),
                    name: Unchanged("F".into()),
                    cake_id: Unchanged(Some(12)),
                }
                .into(),
            ),
            fillings: ActiveHasMany::Append(vec![filling::ActiveModelEx {
                id: Unchanged(14),
                name: Unchanged("FF".into()),
                vendor_id: Unchanged(None),
                ingredients: ActiveHasMany::NotSet,
            }]),
        };

        assert_eq!(cake_ex.into_active_model(), cake_am);
    }
}
