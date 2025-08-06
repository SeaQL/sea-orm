use crate::{
    Condition, ConnectionTrait, DbErr, EntityTrait, Identity, ModelTrait, QueryFilter, Related,
    RelationType, Select, error::*,
};
use async_trait::async_trait;
use sea_query::{
    ColumnRef, DynIden, Expr, ExprTrait, IntoColumnRef, SimpleExpr, TableRef, ValueTuple,
};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

/// Entity, or a Select<Entity>; to be used as parameters in [`LoaderTrait`]
pub trait EntityOrSelect<E: EntityTrait>: Send {
    /// If self is Entity, use Entity::find()
    fn select(self) -> Select<E>;
}

/// This trait implements the Data Loader API
#[async_trait]
pub trait LoaderTrait {
    /// Source model
    type Model: ModelTrait;

    /// Used to eager load has_one relations
    async fn load_one<R, S, C>(&self, stmt: S, db: &C) -> Result<Vec<Option<R::Model>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        <<Self as LoaderTrait>::Model as ModelTrait>::Entity: Related<R>;

    /// Used to eager load has_many relations
    async fn load_many<R, S, C>(&self, stmt: S, db: &C) -> Result<Vec<Vec<R::Model>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        <<Self as LoaderTrait>::Model as ModelTrait>::Entity: Related<R>;

    /// Used to eager load many_to_many relations
    async fn load_many_to_many<R, S, V, C>(
        &self,
        stmt: S,
        via: V,
        db: &C,
    ) -> Result<Vec<Vec<R::Model>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        V: EntityTrait,
        V::Model: Send + Sync,
        <<Self as LoaderTrait>::Model as ModelTrait>::Entity: Related<R>;
}

impl<E> EntityOrSelect<E> for E
where
    E: EntityTrait,
{
    fn select(self) -> Select<E> {
        E::find()
    }
}

impl<E> EntityOrSelect<E> for Select<E>
where
    E: EntityTrait,
{
    fn select(self) -> Select<E> {
        self
    }
}

#[async_trait]
impl<M> LoaderTrait for Vec<M>
where
    M: ModelTrait + Sync,
{
    type Model = M;

    async fn load_one<R, S, C>(&self, stmt: S, db: &C) -> Result<Vec<Option<R::Model>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        <<Self as LoaderTrait>::Model as ModelTrait>::Entity: Related<R>,
    {
        self.as_slice().load_one(stmt, db).await
    }

    async fn load_many<R, S, C>(&self, stmt: S, db: &C) -> Result<Vec<Vec<R::Model>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        <<Self as LoaderTrait>::Model as ModelTrait>::Entity: Related<R>,
    {
        self.as_slice().load_many(stmt, db).await
    }

    async fn load_many_to_many<R, S, V, C>(
        &self,
        stmt: S,
        via: V,
        db: &C,
    ) -> Result<Vec<Vec<R::Model>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        V: EntityTrait,
        V::Model: Send + Sync,
        <<Self as LoaderTrait>::Model as ModelTrait>::Entity: Related<R>,
    {
        self.as_slice().load_many_to_many(stmt, via, db).await
    }
}

#[async_trait]
impl<M> LoaderTrait for &[M]
where
    M: ModelTrait + Sync,
{
    type Model = M;

    async fn load_one<R, S, C>(&self, stmt: S, db: &C) -> Result<Vec<Option<R::Model>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        <<Self as LoaderTrait>::Model as ModelTrait>::Entity: Related<R>,
    {
        // we verify that is HasOne relation
        if <<<Self as LoaderTrait>::Model as ModelTrait>::Entity as Related<R>>::via().is_some() {
            return Err(query_err("Relation is ManytoMany instead of HasOne"));
        }
        let rel_def = <<<Self as LoaderTrait>::Model as ModelTrait>::Entity as Related<R>>::to();
        if rel_def.rel_type == RelationType::HasMany {
            return Err(query_err("Relation is HasMany instead of HasOne"));
        }

        if self.is_empty() {
            return Ok(Vec::new());
        }

        let keys = self
            .iter()
            .map(|model| extract_key(&rel_def.from_col, model))
            .collect::<Result<Vec<_>, _>>()?;

        let condition = prepare_condition(&rel_def.to_tbl, &rel_def.to_col, &keys);

        let stmt = <Select<R> as QueryFilter>::filter(stmt.select(), condition);

        let data = stmt.all(db).await?;

        let hashmap = data.into_iter().try_fold(
            HashMap::<ValueTuple, <R as EntityTrait>::Model>::new(),
            |mut acc, value| {
                extract_key(&rel_def.to_col, &value).map(|key| {
                    acc.insert(key, value);

                    acc
                })
            },
        )?;

        let result: Vec<Option<<R as EntityTrait>::Model>> =
            keys.iter().map(|key| hashmap.get(key).cloned()).collect();

        Ok(result)
    }

    async fn load_many<R, S, C>(&self, stmt: S, db: &C) -> Result<Vec<Vec<R::Model>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        <<Self as LoaderTrait>::Model as ModelTrait>::Entity: Related<R>,
    {
        // we verify that is HasMany relation

        if <<<Self as LoaderTrait>::Model as ModelTrait>::Entity as Related<R>>::via().is_some() {
            return Err(query_err("Relation is ManyToMany instead of HasMany"));
        }
        let rel_def = <<<Self as LoaderTrait>::Model as ModelTrait>::Entity as Related<R>>::to();
        if rel_def.rel_type == RelationType::HasOne {
            return Err(query_err("Relation is HasOne instead of HasMany"));
        }

        if self.is_empty() {
            return Ok(Vec::new());
        }

        let keys = self
            .iter()
            .map(|model| extract_key(&rel_def.from_col, model))
            .collect::<Result<Vec<_>, _>>()?;

        let condition = prepare_condition(&rel_def.to_tbl, &rel_def.to_col, &keys);

        let stmt = <Select<R> as QueryFilter>::filter(stmt.select(), condition);

        let data = stmt.all(db).await?;

        let mut hashmap: HashMap<ValueTuple, Vec<<R as EntityTrait>::Model>> =
            keys.iter()
                .fold(HashMap::new(), |mut acc, key: &ValueTuple| {
                    acc.insert(key.clone(), Vec::new());
                    acc
                });

        for value in data {
            let key = extract_key(&rel_def.to_col, &value)?;

            let vec = hashmap.get_mut(&key).ok_or_else(|| {
                DbErr::RecordNotFound(format!("Loader: failed to find model for {key:?}"))
            })?;

            vec.push(value);
        }

        let result: Vec<Vec<R::Model>> = keys
            .iter()
            .map(|key: &ValueTuple| hashmap.get(key).cloned().unwrap_or_default())
            .collect();

        Ok(result)
    }

    async fn load_many_to_many<R, S, V, C>(
        &self,
        stmt: S,
        via: V,
        db: &C,
    ) -> Result<Vec<Vec<R::Model>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        V: EntityTrait,
        V::Model: Send + Sync,
        <<Self as LoaderTrait>::Model as ModelTrait>::Entity: Related<R>,
    {
        if let Some(via_rel) =
            <<<Self as LoaderTrait>::Model as ModelTrait>::Entity as Related<R>>::via()
        {
            let rel_def =
                <<<Self as LoaderTrait>::Model as ModelTrait>::Entity as Related<R>>::to();
            if rel_def.rel_type != RelationType::HasOne {
                return Err(query_err("Relation to is not HasOne"));
            }

            if !cmp_table_ref(&via_rel.to_tbl, &via.table_ref()) {
                return Err(query_err(format!(
                    "The given via Entity is incorrect: expected: {:?}, given: {:?}",
                    via_rel.to_tbl,
                    via.table_ref()
                )));
            }

            if self.is_empty() {
                return Ok(Vec::new());
            }

            let pkeys = self
                .iter()
                .map(|model| extract_key(&via_rel.from_col, model))
                .collect::<Result<Vec<_>, _>>()?;

            // Map of M::PK -> Vec<R::PK>
            let mut keymap: HashMap<ValueTuple, Vec<ValueTuple>> = Default::default();

            let keys: Vec<ValueTuple> = {
                let condition = prepare_condition(&via_rel.to_tbl, &via_rel.to_col, &pkeys);
                let stmt = V::find().filter(condition);
                let data = stmt.all(db).await?;
                for model in data {
                    let pk = extract_key(&via_rel.to_col, &model)?;
                    let entry = keymap.entry(pk).or_default();

                    let fk = extract_key(&rel_def.from_col, &model)?;
                    entry.push(fk);
                }

                keymap.values().flatten().cloned().collect()
            };

            let condition = prepare_condition(&rel_def.to_tbl, &rel_def.to_col, &keys);

            let stmt = <Select<R> as QueryFilter>::filter(stmt.select(), condition);

            let models = stmt.all(db).await?;

            // Map of R::PK -> R::Model
            let data = models.into_iter().try_fold(
                HashMap::<ValueTuple, <R as EntityTrait>::Model>::new(),
                |mut acc, model| {
                    extract_key(&rel_def.to_col, &model).map(|key| {
                        acc.insert(key, model);

                        acc
                    })
                },
            )?;

            let result: Vec<Vec<R::Model>> = pkeys
                .into_iter()
                .map(|pkey| {
                    let fkeys = keymap.get(&pkey).cloned().unwrap_or_default();

                    let models: Vec<_> = fkeys
                        .into_iter()
                        .filter_map(|fkey| data.get(&fkey).cloned())
                        .collect();

                    models
                })
                .collect();

            Ok(result)
        } else {
            return Err(query_err("Relation is not ManyToMany"));
        }
    }
}

fn cmp_table_ref(left: &TableRef, right: &TableRef) -> bool {
    // not ideal; but
    format!("{left:?}") == format!("{right:?}")
}

fn extract_key<Model>(target_col: &Identity, model: &Model) -> Result<ValueTuple, DbErr>
where
    Model: ModelTrait,
{
    Ok(match target_col {
        Identity::Unary(a) => {
            let a = a.to_string();
            let column_a =
                <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(&a)
                    .map_err(|_| DbErr::Type(format!("Failed at mapping '{a}' to column A:1")))?;
            ValueTuple::One(model.get(column_a))
        }
        Identity::Binary(a, b) => {
            let a = a.to_string();
            let b = b.to_string();
            let column_a =
                <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(&a)
                    .map_err(|_| DbErr::Type(format!("Failed at mapping '{a}' to column A:2")))?;
            let column_b =
                <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(&b)
                    .map_err(|_| DbErr::Type(format!("Failed at mapping '{b}' to column B:2")))?;
            ValueTuple::Two(model.get(column_a), model.get(column_b))
        }
        Identity::Ternary(a, b, c) => {
            let a = a.to_string();
            let b = b.to_string();
            let c = c.to_string();
            let column_a =
                <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(
                    &a.to_string(),
                )
                .map_err(|_| DbErr::Type(format!("Failed at mapping '{a}' to column A:3")))?;
            let column_b =
                <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(
                    &b.to_string(),
                )
                .map_err(|_| DbErr::Type(format!("Failed at mapping '{b}' to column B:3")))?;
            let column_c =
                <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(
                    &c.to_string(),
                )
                .map_err(|_| DbErr::Type(format!("Failed at mapping '{c}' to column C:3")))?;
            ValueTuple::Three(
                model.get(column_a),
                model.get(column_b),
                model.get(column_c),
            )
        }
        Identity::Many(cols) => {
            let mut values = Vec::new();
            for col in cols {
                let col_name = col.to_string();
                let column =
                    <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(
                        &col_name,
                    )
                    .map_err(|_| DbErr::Type(format!("Failed at mapping '{col_name}' to colum")))?;
                values.push(model.get(column))
            }
            ValueTuple::Many(values)
        }
    })
}

fn prepare_condition(table: &TableRef, col: &Identity, keys: &[ValueTuple]) -> Condition {
    let keys = if !keys.is_empty() {
        let set: HashSet<_> = keys.iter().cloned().collect();
        set.into_iter().collect()
    } else {
        Vec::new()
    };

    match col {
        Identity::Unary(column_a) => {
            let column_a = table_column(table, column_a);
            Condition::all().add(Expr::col(column_a).is_in(keys.into_iter().flatten()))
        }
        Identity::Binary(column_a, column_b) => Condition::all().add(
            Expr::tuple([
                SimpleExpr::Column(table_column(table, column_a)),
                SimpleExpr::Column(table_column(table, column_b)),
            ])
            .in_tuples(keys),
        ),
        Identity::Ternary(column_a, column_b, column_c) => Condition::all().add(
            Expr::tuple([
                SimpleExpr::Column(table_column(table, column_a)),
                SimpleExpr::Column(table_column(table, column_b)),
                SimpleExpr::Column(table_column(table, column_c)),
            ])
            .in_tuples(keys),
        ),
        Identity::Many(cols) => {
            let columns = cols
                .iter()
                .map(|col| SimpleExpr::Column(table_column(table, col)));
            Condition::all().add(Expr::tuple(columns).in_tuples(keys))
        }
    }
}

fn table_column(tbl: &TableRef, col: &DynIden) -> ColumnRef {
    (tbl.sea_orm_table().to_owned(), col.clone()).into_column_ref()
}

#[cfg(test)]
mod tests {
    fn cake_model(id: i32) -> sea_orm::tests_cfg::cake::Model {
        let name = match id {
            1 => "apple cake",
            2 => "orange cake",
            3 => "fruit cake",
            4 => "chocolate cake",
            _ => "",
        }
        .to_string();
        sea_orm::tests_cfg::cake::Model { id, name }
    }

    fn fruit_model(id: i32, cake_id: Option<i32>) -> sea_orm::tests_cfg::fruit::Model {
        let name = match id {
            1 => "apple",
            2 => "orange",
            3 => "grape",
            4 => "strawberry",
            _ => "",
        }
        .to_string();
        sea_orm::tests_cfg::fruit::Model { id, name, cake_id }
    }

    fn filling_model(id: i32) -> sea_orm::tests_cfg::filling::Model {
        let name = match id {
            1 => "apple juice",
            2 => "orange jam",
            3 => "chocolate crust",
            4 => "strawberry jam",
            _ => "",
        }
        .to_string();
        sea_orm::tests_cfg::filling::Model {
            id,
            name,
            vendor_id: Some(1),
            ignored_attr: 0,
        }
    }

    fn cake_filling_model(
        cake_id: i32,
        filling_id: i32,
    ) -> sea_orm::tests_cfg::cake_filling::Model {
        sea_orm::tests_cfg::cake_filling::Model {
            cake_id,
            filling_id,
        }
    }

    #[tokio::test]
    async fn test_load_one() {
        use sea_orm::{DbBackend, LoaderTrait, MockDatabase, entity::prelude::*, tests_cfg::*};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[cake_model(1), cake_model(2)]])
            .into_connection();

        let fruits = vec![fruit_model(1, Some(1))];

        let cakes = fruits
            .load_one(cake::Entity::find(), &db)
            .await
            .expect("Should return something");

        assert_eq!(cakes, [Some(cake_model(1))]);
    }

    #[tokio::test]
    async fn test_load_one_same_cake() {
        use sea_orm::{DbBackend, LoaderTrait, MockDatabase, entity::prelude::*, tests_cfg::*};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[cake_model(1), cake_model(2)]])
            .into_connection();

        let fruits = vec![fruit_model(1, Some(1)), fruit_model(2, Some(1))];

        let cakes = fruits
            .load_one(cake::Entity::find(), &db)
            .await
            .expect("Should return something");

        assert_eq!(cakes, [Some(cake_model(1)), Some(cake_model(1))]);
    }

    #[tokio::test]
    async fn test_load_one_empty() {
        use sea_orm::{DbBackend, LoaderTrait, MockDatabase, entity::prelude::*, tests_cfg::*};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[cake_model(1), cake_model(2)]])
            .into_connection();

        let fruits: Vec<fruit::Model> = vec![];

        let cakes = fruits
            .load_one(cake::Entity::find(), &db)
            .await
            .expect("Should return something");

        assert_eq!(cakes, []);
    }

    #[tokio::test]
    async fn test_load_many() {
        use sea_orm::{DbBackend, LoaderTrait, MockDatabase, entity::prelude::*, tests_cfg::*};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[fruit_model(1, Some(1))]])
            .into_connection();

        let cakes = vec![cake_model(1), cake_model(2)];

        let fruits = cakes
            .load_many(fruit::Entity::find(), &db)
            .await
            .expect("Should return something");

        assert_eq!(fruits, [vec![fruit_model(1, Some(1))], vec![]]);
    }

    #[tokio::test]
    async fn test_load_many_same_fruit() {
        use sea_orm::{DbBackend, LoaderTrait, MockDatabase, entity::prelude::*, tests_cfg::*};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[fruit_model(1, Some(1)), fruit_model(2, Some(1))]])
            .into_connection();

        let cakes = vec![cake_model(1), cake_model(2)];

        let fruits = cakes
            .load_many(fruit::Entity::find(), &db)
            .await
            .expect("Should return something");

        assert_eq!(
            fruits,
            [
                vec![fruit_model(1, Some(1)), fruit_model(2, Some(1))],
                vec![]
            ]
        );
    }

    #[tokio::test]
    async fn test_load_many_empty() {
        use sea_orm::{DbBackend, MockDatabase, entity::prelude::*, tests_cfg::*};

        let db = MockDatabase::new(DbBackend::Postgres).into_connection();

        let cakes: Vec<cake::Model> = vec![];

        let fruits = cakes
            .load_many(fruit::Entity::find(), &db)
            .await
            .expect("Should return something");

        let empty_vec: Vec<Vec<fruit::Model>> = vec![];

        assert_eq!(fruits, empty_vec);
    }

    #[tokio::test]
    async fn test_load_many_to_many_base() {
        use sea_orm::{DbBackend, IntoMockRow, LoaderTrait, MockDatabase, tests_cfg::*};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([
                [cake_filling_model(1, 1).into_mock_row()],
                [filling_model(1).into_mock_row()],
            ])
            .into_connection();

        let cakes = vec![cake_model(1)];

        let fillings = cakes
            .load_many_to_many(Filling, CakeFilling, &db)
            .await
            .expect("Should return something");

        assert_eq!(fillings, vec![vec![filling_model(1)]]);
    }

    #[tokio::test]
    async fn test_load_many_to_many_complex() {
        use sea_orm::{DbBackend, IntoMockRow, LoaderTrait, MockDatabase, tests_cfg::*};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([
                [
                    cake_filling_model(1, 1).into_mock_row(),
                    cake_filling_model(1, 2).into_mock_row(),
                    cake_filling_model(1, 3).into_mock_row(),
                    cake_filling_model(2, 1).into_mock_row(),
                    cake_filling_model(2, 2).into_mock_row(),
                ],
                [
                    filling_model(1).into_mock_row(),
                    filling_model(2).into_mock_row(),
                    filling_model(3).into_mock_row(),
                    filling_model(4).into_mock_row(),
                    filling_model(5).into_mock_row(),
                ],
            ])
            .into_connection();

        let cakes = vec![cake_model(1), cake_model(2), cake_model(3)];

        let fillings = cakes
            .load_many_to_many(Filling, CakeFilling, &db)
            .await
            .expect("Should return something");

        assert_eq!(
            fillings,
            vec![
                vec![filling_model(1), filling_model(2), filling_model(3)],
                vec![filling_model(1), filling_model(2)],
                vec![],
            ]
        );
    }

    #[tokio::test]
    async fn test_load_many_to_many_empty() {
        use sea_orm::{DbBackend, IntoMockRow, LoaderTrait, MockDatabase, tests_cfg::*};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([
                [cake_filling_model(1, 1).into_mock_row()],
                [filling_model(1).into_mock_row()],
            ])
            .into_connection();

        let cakes: Vec<cake::Model> = vec![];

        let fillings = cakes
            .load_many_to_many(Filling, CakeFilling, &db)
            .await
            .expect("Should return something");

        let empty_vec: Vec<Vec<filling::Model>> = vec![];

        assert_eq!(fillings, empty_vec);
    }

    #[tokio::test]
    async fn test_load_one_duplicate_keys() {
        use sea_orm::{DbBackend, LoaderTrait, MockDatabase, entity::prelude::*, tests_cfg::*};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[cake_model(1), cake_model(2)]])
            .into_connection();

        let fruits = vec![
            fruit_model(1, Some(1)),
            fruit_model(2, Some(1)),
            fruit_model(3, Some(1)),
            fruit_model(4, Some(1)),
        ];

        let cakes = fruits
            .load_one(cake::Entity::find(), &db)
            .await
            .expect("Should return something");

        assert_eq!(cakes.len(), 4);
        for cake in &cakes {
            assert_eq!(cake, &Some(cake_model(1)));
        }
        let logs = db.into_transaction_log();
        let sql = format!("{:?}", logs[0]);

        let values_count = sql.matches("$1").count();
        assert_eq!(values_count, 1, "Duplicate values were not removed");
    }

    #[tokio::test]
    async fn test_load_many_duplicate_keys() {
        use sea_orm::{DbBackend, LoaderTrait, MockDatabase, entity::prelude::*, tests_cfg::*};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[
                fruit_model(1, Some(1)),
                fruit_model(2, Some(1)),
                fruit_model(3, Some(2)),
            ]])
            .into_connection();

        let cakes = vec![cake_model(1), cake_model(1), cake_model(2), cake_model(2)];

        let fruits = cakes
            .load_many(fruit::Entity::find(), &db)
            .await
            .expect("Should return something");

        assert_eq!(fruits.len(), 4);

        let logs = db.into_transaction_log();
        let sql = format!("{:?}", logs[0]);

        let values_count = sql.matches("$1").count() + sql.matches("$2").count();
        assert_eq!(values_count, 2, "Duplicate values were not removed");
    }
}
