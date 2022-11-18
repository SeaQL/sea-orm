use crate::{DbErr, EntityTrait, ModelTrait, QueryFilter, Select, Related, RelationType, Identity, Condition, Value, ColumnTrait, ConnectionTrait};
use std::{fmt::Debug, str::FromStr, collections::BTreeMap};

#[async_trait::async_trait]
pub trait LoaderTrait {
    type Model: ModelTrait;

    async fn load_one<R, C>(&self, db: &C) -> Result<Vec<Option<R::Model>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        <<R as EntityTrait>::Column as FromStr>::Err: Debug,
        <<Self as LoaderTrait>::Model as ModelTrait>::Entity: Related<R>,
        <<<<Self as LoaderTrait>::Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::Err: Debug;

    async fn load_many<R, C>(&self, db: &C) -> Result<Vec<Vec<R::Model>>, DbErr>
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

    async fn load_one<R, C>(&self, db: &C) -> Result<Vec<Option<R::Model>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        <<R as EntityTrait>::Column as FromStr>::Err: Debug,
        <<Self as LoaderTrait>::Model as ModelTrait>::Entity: Related<R>,
        <<<M as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::Err: Debug,
    {
        let rel_def =
            <<<Self as LoaderTrait>::Model as ModelTrait>::Entity as Related<R>>::to();

        // we verify that is has_one relation
        match (&rel_def).rel_type {
            RelationType::HasOne => (),
            RelationType::HasMany => {
                return Err(DbErr::Type("Relation is HasMany instead of HasOne".into()))
            }
        }

        fn extract_key<Model>(target_col: &Identity, model: &Model) -> Vec<Value>
        where
            Model: ModelTrait,
            <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::Err: Debug,
        {
            match target_col {
                Identity::Unary(a) => {
                    let column_a = <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(&a.to_string()).unwrap();
                    vec![model.get(column_a)]
                },
                Identity::Binary(a, b) => {
                    let column_a = <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(&a.to_string()).unwrap();
                    let column_b = <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(&b.to_string()).unwrap();
                    vec![model.get(column_a), model.get(column_b)]
                },
                Identity::Ternary(a, b, c) => {
                    let column_a = <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(&a.to_string()).unwrap();
                    let column_b = <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(&b.to_string()).unwrap();
                    let column_c = <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(&c.to_string()).unwrap();
                    vec![model.get(column_a), model.get(column_b), model.get(column_c)]
                },
            }
        }

        let keys: Vec<Vec<Value>> = self
            .iter()
            .map(|model: &M| {
                extract_key(&rel_def.from_col, model)
            })
            .collect();

        let condition = match &rel_def.to_col {
            Identity::Unary(a) => {
                let column_a: <M::Entity as EntityTrait>::Column = <<<<Self as LoaderTrait>::Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(&a.to_string()).unwrap();
                Condition::all().add(ColumnTrait::is_in(
                    &column_a,
                    keys.iter().map(|key| key[0].clone()).collect::<Vec<Value>>(),
                ))
            }
            Identity::Binary(a, b) => {
                let column_a: <<<Self as LoaderTrait>::Model as ModelTrait>::Entity as EntityTrait>::Column = <<<<Self as LoaderTrait>::Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(&a.to_string()).unwrap();
                let column_b: <<<Self as LoaderTrait>::Model as ModelTrait>::Entity as EntityTrait>::Column = <<<<Self as LoaderTrait>::Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(&b.to_string()).unwrap();
                // TODO
                // Condition::all().add(
                //     sea_query::Expr::tuple([column_a.to_string(), column_b]).is_in(keys.iter().map(|key| (key[0].clone(), key[1].clone())).collect::<Vec<(Value, Value)>>())
                // )
                // TODO
                Condition::all().add(ColumnTrait::is_in(
                    &column_a,
                    keys.iter().map(|key| key[0].clone()).collect::<Vec<Value>>(),
                ))
            }
            Identity::Ternary(a, b, c) => {
                let column_a = <<<<Self as LoaderTrait>::Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(&a.to_string()).unwrap();
                let column_b = <<<<Self as LoaderTrait>::Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(&b.to_string()).unwrap();
                let column_c = <<<<Self as LoaderTrait>::Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(&c.to_string()).unwrap();
                // TODO
                Condition::all().add(ColumnTrait::is_in(
                    &column_a,
                    keys.iter().map(|key| key[0].clone()).collect::<Vec<Value>>(),
                ))
            }
        };

        let stmt = <R as EntityTrait>::find();

        let stmt = <Select<R> as QueryFilter>::filter(stmt, condition);

        let data = stmt.all(db).await?;

        let mut hashmap: BTreeMap::<String, <R as EntityTrait>::Model> = data
            .into_iter()
            .fold(BTreeMap::<String, <R as EntityTrait>::Model>::new(), |mut acc: BTreeMap::<String, <R as EntityTrait>::Model>, value: <R as EntityTrait>::Model| {
                {
                    let key = extract_key(&rel_def.to_col, &value);

                    acc.insert(format!("{:?}", key), value);
                }

                acc
            });

        let result: Vec<Option<<R as EntityTrait>::Model>> = keys
            .iter()
            .map(|key| {
                let model = hashmap.remove(&format!("{:?}", key));

                model
            })
            .collect();

        Ok(result)
    }

    async fn load_many<R, C>(&self, db: &C) -> Result<Vec<Vec<R::Model>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        <<R as EntityTrait>::Column as FromStr>::Err: Debug,
        <<Self as LoaderTrait>::Model as ModelTrait>::Entity: Related<R>,
        <<<M as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::Err: Debug,
    {
        // we should verify this is a has_many relation
        Ok(vec![])
    }
}
