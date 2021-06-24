use crate::{
    query::combine, DatabaseConnection, EntityTrait, FromQueryResult, Iterable, JsonValue,
    ModelTrait, Paginator, PrimaryKeyToColumn, QueryErr, QueryResult, Select, SelectThree,
    SelectTwo, TypeErr,
};
use sea_query::SelectStatement;
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct Selector<S>
where
    S: SelectorTrait,
{
    query: SelectStatement,
    selector: S,
}

pub trait SelectorTrait {
    type Item: Sized;

    fn from_raw_query_result(res: QueryResult) -> Result<Self::Item, TypeErr>;
}

pub struct SelectModel<M>
where
    M: FromQueryResult,
{
    model: PhantomData<M>,
}

#[derive(Clone, Debug)]
pub struct SelectTwoModel<M, N>
where
    M: FromQueryResult,
    N: FromQueryResult,
{
    model: PhantomData<(M, N)>,
}

#[derive(Clone, Debug)]
pub struct SelectThreeModel<M, N, O>
where
    M: FromQueryResult,
    N: FromQueryResult,
    O: FromQueryResult,
{
    model: PhantomData<(M, N, O)>,
}

impl<M> SelectorTrait for SelectModel<M>
where
    M: FromQueryResult + Sized,
{
    type Item = M;

    fn from_raw_query_result(res: QueryResult) -> Result<Self::Item, TypeErr> {
        M::from_query_result(&res, "")
    }
}

impl<M, N> SelectorTrait for SelectTwoModel<M, N>
where
    M: FromQueryResult + Sized,
    N: FromQueryResult + Sized,
{
    type Item = (M, Option<N>);

    fn from_raw_query_result(res: QueryResult) -> Result<Self::Item, TypeErr> {
        Ok((
            M::from_query_result(&res, combine::SELECT_A)?,
            N::from_query_result_opt(&res, combine::SELECT_B)?,
        ))
    }
}

impl<M, N, O> SelectorTrait for SelectThreeModel<M, N, O>
where
    M: FromQueryResult + Sized,
    N: FromQueryResult + Sized,
    O: FromQueryResult + Sized,
{
    type Item = (M, Option<(N, Option<O>)>);

    fn from_raw_query_result(res: QueryResult) -> Result<Self::Item, TypeErr> {
        Ok((
            M::from_query_result(&res, combine::SELECT_A)?,
            match N::from_query_result_opt(&res, combine::SELECT_B)? {
                Some(n) => Some((n, O::from_query_result_opt(&res, combine::SELECT_C)?)),
                None => None,
            },
        ))
    }
}

impl<E> Select<E>
where
    E: EntityTrait,
{
    pub fn into_model<M>(self) -> Selector<SelectModel<M>>
    where
        M: FromQueryResult,
    {
        Selector {
            query: self.query,
            selector: SelectModel { model: PhantomData },
        }
    }

    #[cfg(feature = "with-json")]
    pub fn into_json(self) -> Selector<SelectModel<JsonValue>> {
        Selector {
            query: self.query,
            selector: SelectModel { model: PhantomData },
        }
    }

    pub async fn one(self, db: &DatabaseConnection) -> Result<Option<E::Model>, QueryErr> {
        self.into_model::<E::Model>().one(db).await
    }

    pub async fn all(self, db: &DatabaseConnection) -> Result<Vec<E::Model>, QueryErr> {
        self.into_model::<E::Model>().all(db).await
    }

    pub fn paginate(
        self,
        db: &DatabaseConnection,
        page_size: usize,
    ) -> Paginator<'_, SelectModel<E::Model>> {
        self.into_model::<E::Model>().paginate(db, page_size)
    }
}

impl<E, F> SelectTwo<E, F>
where
    E: EntityTrait,
    F: EntityTrait,
{
    fn into_model<M, N>(self) -> Selector<SelectTwoModel<M, N>>
    where
        M: FromQueryResult,
        N: FromQueryResult,
    {
        Selector {
            query: self.query,
            selector: SelectTwoModel { model: PhantomData },
        }
    }

    #[cfg(feature = "with-json")]
    pub fn into_json(self) -> Selector<SelectTwoModel<JsonValue, JsonValue>> {
        Selector {
            query: self.query,
            selector: SelectTwoModel { model: PhantomData },
        }
    }

    pub async fn one(
        self,
        db: &DatabaseConnection,
    ) -> Result<Option<(E::Model, Option<F::Model>)>, QueryErr> {
        self.into_model::<E::Model, F::Model>().one(db).await
    }

    pub async fn all(
        self,
        db: &DatabaseConnection,
    ) -> Result<Vec<(E::Model, Vec<F::Model>)>, QueryErr> {
        let rows = self.into_model::<E::Model, F::Model>().all(db).await?;
        Ok(parse_query_result::<E, _>(rows))
    }
}

impl<E, F, G> SelectThree<E, F, G>
where
    E: EntityTrait,
    F: EntityTrait,
    G: EntityTrait,
{
    fn into_model<M, N, O>(self) -> Selector<SelectThreeModel<M, N, O>>
    where
        M: FromQueryResult,
        N: FromQueryResult,
        O: FromQueryResult,
    {
        Selector {
            query: self.query,
            selector: SelectThreeModel { model: PhantomData },
        }
    }

    #[cfg(feature = "with-json")]
    pub fn into_json(self) -> Selector<SelectThreeModel<JsonValue, JsonValue, JsonValue>> {
        Selector {
            query: self.query,
            selector: SelectThreeModel { model: PhantomData },
        }
    }

    pub async fn one(
        self,
        db: &DatabaseConnection,
    ) -> Result<Option<(E::Model, Option<(F::Model, Option<G::Model>)>)>, QueryErr> {
        self.into_model::<E::Model, F::Model, G::Model>()
            .one(db)
            .await
    }

    pub async fn all(
        self,
        db: &DatabaseConnection,
    ) -> Result<Vec<(E::Model, Vec<(F::Model, Vec<G::Model>)>)>, QueryErr> {
        let rows = self
            .into_model::<E::Model, F::Model, G::Model>()
            .all(db)
            .await?;
        let res = parse_query_result::<E, _>(rows)
            .into_iter()
            .map(|(first_tbl, second_tbl)| {
                let second_tbl_gp = parse_query_result::<F, _>(second_tbl);
                (first_tbl, second_tbl_gp)
            })
            .collect();
        Ok(res)
    }
}

impl<S> Selector<S>
where
    S: SelectorTrait,
{
    pub async fn one(mut self, db: &DatabaseConnection) -> Result<Option<S::Item>, QueryErr> {
        let builder = db.get_query_builder_backend();
        self.query.limit(1);
        let row = db.query_one(builder.build(&self.query)).await?;
        match row {
            Some(row) => Ok(Some(S::from_raw_query_result(row)?)),
            None => Ok(None),
        }
    }

    pub async fn all(self, db: &DatabaseConnection) -> Result<Vec<S::Item>, QueryErr> {
        let builder = db.get_query_builder_backend();
        let rows = db.query_all(builder.build(&self.query)).await?;
        let mut models = Vec::new();
        for row in rows.into_iter() {
            models.push(S::from_raw_query_result(row)?);
        }
        Ok(models)
    }

    pub fn paginate(self, db: &DatabaseConnection, page_size: usize) -> Paginator<'_, S> {
        Paginator {
            query: self.query,
            page: 0,
            page_size,
            db,
            selector: PhantomData,
        }
    }
}

fn parse_query_result<L, R>(rows: Vec<(L::Model, Option<R>)>) -> Vec<(L::Model, Vec<R>)>
where
    L: EntityTrait,
{
    let mut acc: Vec<(L::Model, Vec<R>)> = Vec::new();
    for (l, r) in rows {
        if let Some((last_l, last_r)) = acc.last_mut() {
            let mut same_l = true;
            for pk_col in <L::PrimaryKey as Iterable>::iter() {
                let col = pk_col.into_column();
                let val = l.get(col);
                let last_val = last_l.get(col);
                if !val.eq(&last_val) {
                    same_l = false;
                    break;
                }
            }
            if same_l && r.is_some() {
                last_r.push(r.unwrap());
                continue;
            }
        }
        if r.is_some() {
            acc.push((l, vec![r.unwrap()]));
        } else {
            acc.push((l, vec![]));
        }
    }
    acc
}

#[cfg(test)]
#[cfg(feature = "mock")]
mod tests {
    use crate::combine::{SELECT_A, SELECT_B, SELECT_C};
    use crate::entity::prelude::*;
    use crate::tests_cfg::*;
    use crate::{DatabaseConnection, Iterable, MockDatabase, QueryErr, Transaction};
    use sea_query::{Alias, Expr, Order, SelectStatement};

    fn setup_select() -> (DatabaseConnection, Vec<Vec<cake::Model>>) {
        let case1 = vec![
            cake::Model {
                id: 1,
                name: "New York Cheese".into(),
            },
            cake::Model {
                id: 2,
                name: "Chocolate Forest".into(),
            },
        ];

        let case2 = vec![cake::Model {
            id: 3,
            name: "Tiramisu".into(),
        }];

        let case3 = Vec::new();

        let db = MockDatabase::new()
            .append_query_results(vec![case1.clone(), case2.clone(), case3.clone()])
            .into_connection();

        (db, vec![case1, case2, case3])
    }

    #[async_std::test]
    async fn select_one() -> Result<(), QueryErr> {
        let (db, cases) = setup_select();

        assert_eq!(
            cake::Entity::find().one(&db).await?,
            Some(cases[0][0].clone())
        );
        assert_eq!(
            cake::Entity::find().one(&db).await?,
            Some(cases[1][0].clone())
        );
        assert_eq!(cake::Entity::find().one(&db).await?, None);

        let select = SelectStatement::new()
            .exprs(vec![
                Expr::tbl(cake::Entity, cake::Column::Id),
                Expr::tbl(cake::Entity, cake::Column::Name),
            ])
            .from(cake::Entity)
            .limit(1)
            .to_owned();

        let query_builder = db.get_query_builder_backend();
        let stmts = vec![
            query_builder.build(&select),
            query_builder.build(&select),
            query_builder.build(&select),
        ];

        let mut mocker = db.as_mock_connection().get_mocker_mutex().lock().unwrap();

        assert_eq!(mocker.drain_transaction_log(), Transaction::wrap(stmts));
        Ok(())
    }

    #[async_std::test]
    async fn select_all() -> Result<(), QueryErr> {
        let (db, cases) = setup_select();

        assert_eq!(cake::Entity::find().all(&db).await?, cases[0].clone());
        assert_eq!(cake::Entity::find().all(&db).await?, cases[1].clone());
        assert_eq!(cake::Entity::find().all(&db).await?, vec![]);

        let select = SelectStatement::new()
            .exprs(vec![
                Expr::tbl(cake::Entity, cake::Column::Id),
                Expr::tbl(cake::Entity, cake::Column::Name),
            ])
            .from(cake::Entity)
            .to_owned();

        let query_builder = db.get_query_builder_backend();
        let stmts = vec![
            query_builder.build(&select),
            query_builder.build(&select),
            query_builder.build(&select),
        ];

        let mut mocker = db.as_mock_connection().get_mocker_mutex().lock().unwrap();

        assert_eq!(mocker.drain_transaction_log(), Transaction::wrap(stmts));
        Ok(())
    }

    fn setup_select_two() -> (
        DatabaseConnection,
        Vec<Vec<(cake::Model, Vec<fruit::Model>)>>,
    ) {
        let case1 = vec![
            (
                cake::Model {
                    id: 1,
                    name: "New York Cheese".into(),
                },
                vec![
                    fruit::Model {
                        id: 10,
                        name: "Blueberry".into(),
                        cake_id: Some(1),
                    },
                    fruit::Model {
                        id: 11,
                        name: "Rasberry".into(),
                        cake_id: Some(1),
                    },
                    fruit::Model {
                        id: 12,
                        name: "Apple".into(),
                        cake_id: Some(1),
                    },
                ],
            ),
            (
                cake::Model {
                    id: 2,
                    name: "Chocolate Forest".into(),
                },
                vec![
                    fruit::Model {
                        id: 20,
                        name: "Strawberry".into(),
                        cake_id: Some(2),
                    },
                    fruit::Model {
                        id: 21,
                        name: "King Strawberry".into(),
                        cake_id: Some(2),
                    },
                ],
            ),
        ];

        let case2 = vec![(
            cake::Model {
                id: 3,
                name: "Tiramisu".into(),
            },
            vec![],
        )];

        let case3 = Vec::new();

        let map_mock_row =
            |rows: &Vec<(cake::Model, Vec<fruit::Model>)>| -> Vec<(cake::Model, Option<fruit::Model>)> {
                rows.clone()
                    .into_iter()
                    .map(|(cake, second_vec)| {
                        if second_vec.is_empty() {
                            vec![(cake.clone(), None)]
                        } else {
                            second_vec
                                .into_iter()
                                .map(|fruit| (cake.clone(), Some(fruit)))
                                .collect::<Vec<_>>()
                        }
                    })
                    .flatten()
                    .collect()
            };

        let db = MockDatabase::new()
            .append_query_results(vec![
                map_mock_row(&case1),
                map_mock_row(&case2),
                map_mock_row(&case3),
            ])
            .into_connection();

        (db, vec![case1, case2, case3])
    }

    #[async_std::test]
    async fn select_two_one() -> Result<(), QueryErr> {
        let (db, cases) = setup_select_two();

        let (case1_l, case1_r) = &cases[0][0];
        assert_eq!(
            cake::Entity::find()
                .left_join_and_select(fruit::Entity)
                .one(&db)
                .await?,
            Some((case1_l.clone(), Some(case1_r[0].clone())))
        );

        let (case2_l, _) = &cases[1][0];
        assert_eq!(
            cake::Entity::find()
                .left_join_and_select(fruit::Entity)
                .one(&db)
                .await?,
            Some((case2_l.clone(), None))
        );

        assert_eq!(
            cake::Entity::find()
                .left_join_and_select(fruit::Entity)
                .one(&db)
                .await?,
            None
        );

        let mut select = SelectStatement::new()
            .from(cake::Entity)
            .left_join(
                fruit::Entity,
                Expr::tbl(cake::Entity, cake::Column::Id)
                    .equals(fruit::Entity, fruit::Column::CakeId),
            )
            .order_by_expr(Expr::tbl(cake::Entity, cake::Column::Id).into(), Order::Asc)
            .limit(1)
            .to_owned();
        for col in cake::Column::iter() {
            select.expr_as(
                Expr::tbl(cake::Entity, col),
                Alias::new(&format!("{}{}", SELECT_A, col.to_string())),
            );
        }
        for col in fruit::Column::iter() {
            select.expr_as(
                Expr::tbl(fruit::Entity, col),
                Alias::new(&format!("{}{}", SELECT_B, col.to_string())),
            );
        }

        let query_builder = db.get_query_builder_backend();
        let stmts = vec![
            query_builder.build(&select),
            query_builder.build(&select),
            query_builder.build(&select),
        ];

        let mut mocker = db.as_mock_connection().get_mocker_mutex().lock().unwrap();

        assert_eq!(mocker.drain_transaction_log(), Transaction::wrap(stmts));
        Ok(())
    }

    #[async_std::test]
    async fn select_two_all() -> Result<(), QueryErr> {
        let (db, cases) = setup_select_two();

        assert_eq!(
            cake::Entity::find()
                .left_join_and_select(fruit::Entity)
                .all(&db)
                .await?,
            cases[0]
        );

        assert_eq!(
            cake::Entity::find()
                .left_join_and_select(fruit::Entity)
                .all(&db)
                .await?,
            cases[1]
        );

        assert_eq!(
            cake::Entity::find()
                .left_join_and_select(fruit::Entity)
                .all(&db)
                .await?,
            cases[2]
        );

        let mut select = SelectStatement::new()
            .from(cake::Entity)
            .left_join(
                fruit::Entity,
                Expr::tbl(cake::Entity, cake::Column::Id)
                    .equals(fruit::Entity, fruit::Column::CakeId),
            )
            .order_by_expr(Expr::tbl(cake::Entity, cake::Column::Id).into(), Order::Asc)
            .to_owned();
        for col in cake::Column::iter() {
            select.expr_as(
                Expr::tbl(cake::Entity, col),
                Alias::new(&format!("{}{}", SELECT_A, col.to_string())),
            );
        }
        for col in fruit::Column::iter() {
            select.expr_as(
                Expr::tbl(fruit::Entity, col),
                Alias::new(&format!("{}{}", SELECT_B, col.to_string())),
            );
        }

        let query_builder = db.get_query_builder_backend();
        let stmts = vec![
            query_builder.build(&select),
            query_builder.build(&select),
            query_builder.build(&select),
        ];

        let mut mocker = db.as_mock_connection().get_mocker_mutex().lock().unwrap();

        assert_eq!(mocker.drain_transaction_log(), Transaction::wrap(stmts));
        Ok(())
    }

    fn setup_select_three() -> (
        DatabaseConnection,
        Vec<Vec<(cake::Model, Vec<(fruit::Model, Vec<vendor::Model>)>)>>,
    ) {
        let case1 = vec![
            (
                cake::Model {
                    id: 1,
                    name: "New York Cheese".into(),
                },
                vec![
                    (
                        fruit::Model {
                            id: 10,
                            name: "Blueberry".into(),
                            cake_id: Some(1),
                        },
                        vec![vendor::Model {
                            id: 100,
                            name: "".into(),
                            fruit_id: Some(10),
                        }],
                    ),
                    (
                        fruit::Model {
                            id: 11,
                            name: "Rasberry".into(),
                            cake_id: Some(1),
                        },
                        vec![vendor::Model {
                            id: 101,
                            name: "".into(),
                            fruit_id: Some(11),
                        }],
                    ),
                    (
                        fruit::Model {
                            id: 12,
                            name: "Apple".into(),
                            cake_id: Some(1),
                        },
                        vec![],
                    ),
                ],
            ),
            (
                cake::Model {
                    id: 2,
                    name: "Chocolate Forest".into(),
                },
                vec![
                    (
                        fruit::Model {
                            id: 20,
                            name: "Strawberry".into(),
                            cake_id: Some(2),
                        },
                        vec![],
                    ),
                    (
                        fruit::Model {
                            id: 21,
                            name: "King Strawberry".into(),
                            cake_id: Some(2),
                        },
                        vec![],
                    ),
                ],
            ),
        ];

        let case2 = vec![(
            cake::Model {
                id: 3,
                name: "Tiramisu".into(),
            },
            vec![(
                fruit::Model {
                    id: 30,
                    name: "Blueberry".into(),
                    cake_id: Some(3),
                },
                vec![],
            )],
        )];

        let case3 = vec![(
            cake::Model {
                id: 4,
                name: "Vanilla Cake".into(),
            },
            vec![],
        )];

        let case4 = Vec::new();

        let map_mock_row = |rows: &Vec<(cake::Model, Vec<(fruit::Model, Vec<vendor::Model>)>)>| -> Vec<(
            cake::Model,
            Option<(fruit::Model, Option<vendor::Model>)>,
        )> {
            rows.clone()
                .into_iter()
                .map(|(cake, second_vec)| {
                    if second_vec.is_empty() {
                        vec![(cake.clone(), None)]
                    } else {
                        second_vec
                            .into_iter()
                            .map(|(fruit, third_vec)| {
                                if third_vec.is_empty() {
                                    vec![(cake.clone(), Some((fruit.clone(), None)))]
                                } else {
                                    third_vec
                                        .into_iter()
                                        .map(|vendor| {
                                            (cake.clone(), Some((fruit.clone(), Some(vendor))))
                                        })
                                        .collect::<Vec<_>>()
                                }
                            })
                            .flatten()
                            .collect::<Vec<_>>()
                    }
                })
                .flatten()
                .collect::<Vec<_>>()
        };

        let db = MockDatabase::new()
            .append_query_results(vec![
                map_mock_row(&case1),
                map_mock_row(&case2),
                map_mock_row(&case3),
                map_mock_row(&case4),
            ])
            .into_connection();

        (db, vec![case1, case2, case3, case4])
    }

    #[async_std::test]
    async fn select_three_one() -> Result<(), QueryErr> {
        let (db, cases) = setup_select_three();

        let (case1_cake, case1_fv) = &cases[0][0];
        let (case1_fruit, case1_vendors) = &case1_fv[0];
        let case1_vendor = &case1_vendors[0];
        assert_eq!(
            cake::Entity::find()
                .left_join_and_select(fruit::Entity)
                .left_join_and_select(vendor::Entity)
                .one(&db)
                .await?,
            Some((
                case1_cake.clone(),
                Some((case1_fruit.clone(), Some(case1_vendor.clone())))
            ))
        );

        let (case2_cake, case2_fv) = &cases[1][0];
        let (case2_fruit, _) = &case2_fv[0];
        assert_eq!(
            cake::Entity::find()
                .left_join_and_select(fruit::Entity)
                .left_join_and_select(vendor::Entity)
                .one(&db)
                .await?,
            Some((case2_cake.clone(), Some((case2_fruit.clone(), None))))
        );

        let (case3_cake, _) = &cases[2][0];
        assert_eq!(
            cake::Entity::find()
                .left_join_and_select(fruit::Entity)
                .left_join_and_select(vendor::Entity)
                .one(&db)
                .await?,
            Some((case3_cake.clone(), None))
        );

        assert_eq!(
            cake::Entity::find()
                .left_join_and_select(fruit::Entity)
                .left_join_and_select(vendor::Entity)
                .one(&db)
                .await?,
            None
        );

        let mut select = SelectStatement::new()
            .from(cake::Entity)
            .left_join(
                fruit::Entity,
                Expr::tbl(cake::Entity, cake::Column::Id)
                    .equals(fruit::Entity, fruit::Column::CakeId),
            )
            .left_join(
                vendor::Entity,
                Expr::tbl(fruit::Entity, fruit::Column::Id)
                    .equals(vendor::Entity, vendor::Column::FruitId),
            )
            .order_by_expr(Expr::tbl(cake::Entity, cake::Column::Id).into(), Order::Asc)
            .order_by_expr(
                Expr::tbl(fruit::Entity, fruit::Column::Id).into(),
                Order::Asc,
            )
            .limit(1)
            .to_owned();
        for col in cake::Column::iter() {
            select.expr_as(
                Expr::tbl(cake::Entity, col),
                Alias::new(&format!("{}{}", SELECT_A, col.to_string())),
            );
        }
        for col in fruit::Column::iter() {
            select.expr_as(
                Expr::tbl(fruit::Entity, col),
                Alias::new(&format!("{}{}", SELECT_B, col.to_string())),
            );
        }
        for col in vendor::Column::iter() {
            select.expr_as(
                Expr::tbl(vendor::Entity, col),
                Alias::new(&format!("{}{}", SELECT_C, col.to_string())),
            );
        }

        let query_builder = db.get_query_builder_backend();
        let stmts = vec![
            query_builder.build(&select),
            query_builder.build(&select),
            query_builder.build(&select),
            query_builder.build(&select),
        ];

        let mut mocker = db.as_mock_connection().get_mocker_mutex().lock().unwrap();

        assert_eq!(mocker.drain_transaction_log(), Transaction::wrap(stmts));
        Ok(())
    }

    #[async_std::test]
    async fn select_three_all() -> Result<(), QueryErr> {
        let (db, cases) = setup_select_three();

        assert_eq!(
            cake::Entity::find()
                .left_join_and_select(fruit::Entity)
                .left_join_and_select(vendor::Entity)
                .all(&db)
                .await?,
            cases[0]
        );

        assert_eq!(
            cake::Entity::find()
                .left_join_and_select(fruit::Entity)
                .left_join_and_select(vendor::Entity)
                .all(&db)
                .await?,
            cases[1]
        );

        assert_eq!(
            cake::Entity::find()
                .left_join_and_select(fruit::Entity)
                .left_join_and_select(vendor::Entity)
                .all(&db)
                .await?,
            cases[2]
        );

        assert_eq!(
            cake::Entity::find()
                .left_join_and_select(fruit::Entity)
                .left_join_and_select(vendor::Entity)
                .all(&db)
                .await?,
            cases[3]
        );

        let mut select = SelectStatement::new()
            .from(cake::Entity)
            .left_join(
                fruit::Entity,
                Expr::tbl(cake::Entity, cake::Column::Id)
                    .equals(fruit::Entity, fruit::Column::CakeId),
            )
            .left_join(
                vendor::Entity,
                Expr::tbl(fruit::Entity, fruit::Column::Id)
                    .equals(vendor::Entity, vendor::Column::FruitId),
            )
            .order_by_expr(Expr::tbl(cake::Entity, cake::Column::Id).into(), Order::Asc)
            .order_by_expr(
                Expr::tbl(fruit::Entity, fruit::Column::Id).into(),
                Order::Asc,
            )
            .to_owned();
        for col in cake::Column::iter() {
            select.expr_as(
                Expr::tbl(cake::Entity, col),
                Alias::new(&format!("{}{}", SELECT_A, col.to_string())),
            );
        }
        for col in fruit::Column::iter() {
            select.expr_as(
                Expr::tbl(fruit::Entity, col),
                Alias::new(&format!("{}{}", SELECT_B, col.to_string())),
            );
        }
        for col in vendor::Column::iter() {
            select.expr_as(
                Expr::tbl(vendor::Entity, col),
                Alias::new(&format!("{}{}", SELECT_C, col.to_string())),
            );
        }

        let query_builder = db.get_query_builder_backend();
        let stmts = vec![
            query_builder.build(&select),
            query_builder.build(&select),
            query_builder.build(&select),
            query_builder.build(&select),
        ];

        let mut mocker = db.as_mock_connection().get_mocker_mutex().lock().unwrap();

        assert_eq!(mocker.drain_transaction_log(), Transaction::wrap(stmts));
        Ok(())
    }
}
