use crate::{
    ColumnTrait, Condition, ConnectionTrait, DbErr, EntityTrait, Identity, ModelTrait, QueryFilter,
    Related, RelationType, Select,
};
use async_trait::async_trait;
use sea_query::{Expr, IntoColumnRef, SimpleExpr, ValueTuple};
use std::{collections::BTreeMap, fmt::Debug, str::FromStr};

/// A trait for basic Dataloader
#[async_trait]
pub trait LoaderTrait {
    /// Source model
    type Model: ModelTrait;

    ///
    /// Used to eager load has_one relations
    ///
    async fn load_one<R, C>(&self, stmt: Select<R>, db: &C) -> Result<Vec<Option<R::Model>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        <<R as EntityTrait>::Column as FromStr>::Err: Debug,
        <<Self as LoaderTrait>::Model as ModelTrait>::Entity: Related<R>,
        <<<<Self as LoaderTrait>::Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::Err: Debug;

    ///
    ///  Used to eager load has_many relations
    ///
    async fn load_many<R, C>(&self, stmt: Select<R>, db: &C) -> Result<Vec<Vec<R::Model>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        <<R as EntityTrait>::Column as FromStr>::Err: Debug,
        <<Self as LoaderTrait>::Model as ModelTrait>::Entity: Related<R>,
        <<<<Self as LoaderTrait>::Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::Err: Debug;
}

#[async_trait::async_trait]
impl<M> LoaderTrait for Vec<M>
where
    M: ModelTrait,
    Vec<M>: Sync,
{
    type Model = M;

    async fn load_one<R, C>(&self, stmt: Select<R>, db: &C) -> Result<Vec<Option<R::Model>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        <<R as EntityTrait>::Column as FromStr>::Err: Debug,
        <<Self as LoaderTrait>::Model as ModelTrait>::Entity: Related<R>,
        <<<M as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::Err: Debug,
    {
        let rel_def = <<<Self as LoaderTrait>::Model as ModelTrait>::Entity as Related<R>>::to();

        // we verify that is has_one relation
        match (rel_def).rel_type {
            RelationType::HasOne => (),
            RelationType::HasMany => {
                return Err(DbErr::Type("Relation is HasMany instead of HasOne".into()))
            }
        }

        let keys: Vec<ValueTuple> = self
            .iter()
            .map(|model: &M| extract_key(&rel_def.from_col, model))
            .collect();

        let condition = prepare_condition::<<R as EntityTrait>::Model>(&rel_def.to_col, &keys);

        let stmt = <Select<R> as QueryFilter>::filter(stmt, condition);

        let data = stmt.all(db).await?;

        let mut hashmap: BTreeMap<String, <R as EntityTrait>::Model> = data.into_iter().fold(
            BTreeMap::<String, <R as EntityTrait>::Model>::new(),
            |mut acc: BTreeMap<String, <R as EntityTrait>::Model>,
             value: <R as EntityTrait>::Model| {
                {
                    let key = extract_key(&rel_def.to_col, &value);

                    acc.insert(format!("{:?}", key), value);
                }

                acc
            },
        );

        let result: Vec<Option<<R as EntityTrait>::Model>> = keys
            .iter()
            .map(|key| hashmap.remove(&format!("{:?}", key)))
            .collect();

        Ok(result)
    }

    async fn load_many<R, C>(&self, stmt: Select<R>, db: &C) -> Result<Vec<Vec<R::Model>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        <<R as EntityTrait>::Column as FromStr>::Err: Debug,
        <<Self as LoaderTrait>::Model as ModelTrait>::Entity: Related<R>,
        <<<M as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::Err: Debug,
    {
        let rel_def = <<<Self as LoaderTrait>::Model as ModelTrait>::Entity as Related<R>>::to();

        // we verify that is has_many relation
        match (rel_def).rel_type {
            RelationType::HasMany => (),
            RelationType::HasOne => {
                return Err(DbErr::Type("Relation is HasOne instead of HasMany".into()))
            }
        }

        let keys: Vec<ValueTuple> = self
            .iter()
            .map(|model: &M| extract_key(&rel_def.from_col, model))
            .collect();

        let condition = prepare_condition::<<R as EntityTrait>::Model>(&rel_def.to_col, &keys);

        let stmt = <Select<R> as QueryFilter>::filter(stmt, condition);

        let data = stmt.all(db).await?;

        let mut hashmap: BTreeMap<String, Vec<<R as EntityTrait>::Model>> =
            keys.iter()
                .fold(BTreeMap::new(), |mut acc, key: &ValueTuple| {
                    acc.insert(format!("{:?}", key), Vec::new());

                    acc
                });

        data.into_iter()
            .for_each(|value: <R as EntityTrait>::Model| {
                let key = extract_key(&rel_def.to_col, &value);

                let vec = hashmap
                    .get_mut(&format!("{:?}", key))
                    .expect("Failed at finding key on hashmap");

                vec.push(value);
            });

        let result: Vec<Vec<R::Model>> = keys
            .iter()
            .map(|key: &ValueTuple| {
                hashmap
                    .remove(&format!("{:?}", key))
                    .expect("Failed to convert key to owned")
            })
            .collect();

        Ok(result)
    }
}

fn extract_key<Model>(target_col: &Identity, model: &Model) -> ValueTuple
where
    Model: ModelTrait,
    <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::Err: Debug,
{
    match target_col {
        Identity::Unary(a) => {
            let column_a =
                <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(
                    &a.to_string(),
                )
                .expect("Failed at mapping string to column A:1");
            ValueTuple::One(model.get(column_a))
        }
        Identity::Binary(a, b) => {
            let column_a =
                <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(
                    &a.to_string(),
                )
                .expect("Failed at mapping string to column A:2");
            let column_b =
                <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(
                    &b.to_string(),
                )
                .expect("Failed at mapping string to column B:2");
            ValueTuple::Two(model.get(column_a), model.get(column_b))
        }
        Identity::Ternary(a, b, c) => {
            let column_a =
                <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(
                    &a.to_string(),
                )
                .expect("Failed at mapping string to column A:3");
            let column_b =
                <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(
                    &b.to_string(),
                )
                .expect("Failed at mapping string to column B:3");
            let column_c =
                <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(
                    &c.to_string(),
                )
                .expect("Failed at mapping string to column C:3");
            ValueTuple::Three(
                model.get(column_a),
                model.get(column_b),
                model.get(column_c),
            )
        }
    }
}

fn prepare_condition<M>(col: &Identity, keys: &[ValueTuple]) -> Condition
where
    M: ModelTrait,
    <<<M as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::Err: Debug,
{
    match col {
        Identity::Unary(column_a) => {
            let column_a: <M::Entity as EntityTrait>::Column =
                <<M::Entity as EntityTrait>::Column as FromStr>::from_str(&column_a.to_string())
                    .expect("Failed at mapping string to column *A:1");
            Condition::all().add(ColumnTrait::is_in(
                &column_a,
                keys.iter().cloned().flatten(),
            ))
        }
        Identity::Binary(column_a, column_b) => {
            let column_a: <M::Entity as EntityTrait>::Column =
                <<M::Entity as EntityTrait>::Column as FromStr>::from_str(&column_a.to_string())
                    .expect("Failed at mapping string to column *A:2");
            let column_b: <M::Entity as EntityTrait>::Column =
                <<M::Entity as EntityTrait>::Column as FromStr>::from_str(&column_b.to_string())
                    .expect("Failed at mapping string to column *B:2");
            Condition::all().add(
                Expr::tuple([
                    SimpleExpr::Column(column_a.into_column_ref()),
                    SimpleExpr::Column(column_b.into_column_ref()),
                ])
                .in_tuples(keys.iter().cloned()),
            )
        }
        Identity::Ternary(column_a, column_b, column_c) => {
            let column_a: <M::Entity as EntityTrait>::Column =
                <<M::Entity as EntityTrait>::Column as FromStr>::from_str(&column_a.to_string())
                    .expect("Failed at mapping string to column *A:3");
            let column_b: <M::Entity as EntityTrait>::Column =
                <<M::Entity as EntityTrait>::Column as FromStr>::from_str(&column_b.to_string())
                    .expect("Failed at mapping string to column *B:3");
            let column_c: <M::Entity as EntityTrait>::Column =
                <<M::Entity as EntityTrait>::Column as FromStr>::from_str(&column_c.to_string())
                    .expect("Failed at mapping string to column *C:3");
            Condition::all().add(
                Expr::tuple([
                    SimpleExpr::Column(column_a.into_column_ref()),
                    SimpleExpr::Column(column_b.into_column_ref()),
                    SimpleExpr::Column(column_c.into_column_ref()),
                ])
                .in_tuples(keys.iter().cloned()),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]

    async fn test_load_one() {
        use crate::{
            entity::prelude::*, tests_cfg::*, DbBackend, IntoMockRow, LoaderTrait, MockDatabase,
        };

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results(vec![vec![
                cake::Model {
                    id: 1,
                    name: "New York Cheese".to_owned(),
                }
                .into_mock_row(),
                cake::Model {
                    id: 2,
                    name: "London Cheese".to_owned(),
                }
                .into_mock_row(),
            ]])
            .into_connection();

        let fruits = vec![fruit::Model {
            id: 1,
            name: "Apple".to_owned(),
            cake_id: Some(1),
        }];

        let cakes = fruits
            .load_one(cake::Entity::find(), &db)
            .await
            .expect("Should return something");

        assert_eq!(
            cakes,
            vec![Some(cake::Model {
                id: 1,
                name: "New York Cheese".to_owned(),
            })]
        );
    }

    #[tokio::test]

    async fn test_load_many() {
        use crate::{
            entity::prelude::*, tests_cfg::*, DbBackend, IntoMockRow, LoaderTrait, MockDatabase,
        };

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results(vec![vec![fruit::Model {
                id: 1,
                name: "Apple".to_owned(),
                cake_id: Some(1),
            }
            .into_mock_row()]])
            .into_connection();

        let cakes = vec![
            cake::Model {
                id: 1,
                name: "New York Cheese".to_owned(),
            },
            cake::Model {
                id: 2,
                name: "London Cheese".to_owned(),
            },
        ];

        let fruits = cakes
            .load_many(fruit::Entity::find(), &db)
            .await
            .expect("Should return something");

        assert_eq!(
            fruits,
            vec![
                vec![fruit::Model {
                    id: 1,
                    name: "Apple".to_owned(),
                    cake_id: Some(1),
                }],
                vec![]
            ]
        );
    }
}
