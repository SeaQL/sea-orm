use crate::{
    EntityTrait, Iterable, ModelTrait, PrimaryKeyArity, PrimaryKeyToColumn, PrimaryKeyTrait,
};
use sea_query::Value;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub(super) fn consolidate_query_result<L, R>(
    rows: Vec<(L::Model, Option<R::Model>)>,
) -> Vec<(L::Model, Vec<R::Model>)>
where
    L: EntityTrait,
    R: EntityTrait,
{
    // given that ARITY is a compile-time const, I hope the other branches can be eliminated as dead code
    match <<L::PrimaryKey as PrimaryKeyTrait>::ValueType as PrimaryKeyArity>::ARITY {
        1 => consolidate_query_result_of::<L, R, _>(rows, unit_pk::<L>()),
        2 => consolidate_query_result_of::<L, R, _>(rows, pair_pk::<L>()),
        _ => consolidate_query_result_of::<L, R, _>(rows, tuple_pk::<L>()),
    }
}

pub(super) fn consolidate_query_result_tee<L, M, R>(
    rows: Vec<(L::Model, Option<M::Model>, Option<R::Model>)>,
) -> Vec<(L::Model, Vec<M::Model>, Vec<R::Model>)>
where
    L: EntityTrait,
    M: EntityTrait,
    R: EntityTrait,
{
    match <<L::PrimaryKey as PrimaryKeyTrait>::ValueType as PrimaryKeyArity>::ARITY {
        1 => consolidate_query_result_of_tee::<L, M, R, _>(rows, unit_pk::<L>()),
        2 => consolidate_query_result_of_tee::<L, M, R, _>(rows, pair_pk::<L>()),
        _ => consolidate_query_result_of_tee::<L, M, R, _>(rows, tuple_pk::<L>()),
    }
}

pub(super) fn consolidate_query_result_chain<L, M, R>(
    rows: Vec<(L::Model, Option<M::Model>, Option<R::Model>)>,
) -> Vec<(L::Model, Vec<(M::Model, Vec<R::Model>)>)>
where
    L: EntityTrait,
    M: EntityTrait,
    R: EntityTrait,
{
    match <<L::PrimaryKey as PrimaryKeyTrait>::ValueType as PrimaryKeyArity>::ARITY {
        1 => consolidate_query_result_of_chain::<L, M, R, _>(rows, unit_pk::<L>()),
        2 => consolidate_query_result_of_chain::<L, M, R, _>(rows, pair_pk::<L>()),
        _ => consolidate_query_result_of_chain::<L, M, R, _>(rows, tuple_pk::<L>()),
    }
}

fn retain_unique_models<L>(rows: Vec<L::Model>) -> Vec<L::Model>
where
    L: EntityTrait,
{
    match <<L::PrimaryKey as PrimaryKeyTrait>::ValueType as PrimaryKeyArity>::ARITY {
        1 => retain_unique_models_of::<L, _>(rows, unit_pk::<L>()),
        2 => retain_unique_models_of::<L, _>(rows, pair_pk::<L>()),
        _ => retain_unique_models_of::<L, _>(rows, tuple_pk::<L>()),
    }
}

// This generic here is that we need to support composite primary key,
// and that we don't want to penalize the unit pk which is the most common.
trait ModelKey<E: EntityTrait> {
    type Type: Hash + PartialEq + Eq;
    fn get(&self, model: &E::Model) -> Self::Type;
}

// This could have been an array of [E::Column; <E::PrimaryKey as PrimaryKeyTrait>::ARITY], but it still doesn't compile
struct UnitPk<E: EntityTrait>(E::Column);
struct PairPk<E: EntityTrait>(E::Column, E::Column);
struct TuplePk<E: EntityTrait>(Vec<E::Column>);

#[allow(clippy::unwrap_used)]
fn unit_pk<E: EntityTrait>() -> UnitPk<E> {
    let col = <E::PrimaryKey as Iterable>::iter()
        .next()
        .unwrap()
        .into_column();
    UnitPk(col)
}

#[allow(clippy::unwrap_used)]
fn pair_pk<E: EntityTrait>() -> PairPk<E> {
    let mut iter = <E::PrimaryKey as Iterable>::iter();
    let col1 = iter.next().unwrap().into_column();
    let col2 = iter.next().unwrap().into_column();
    PairPk(col1, col2)
}

fn tuple_pk<E: EntityTrait>() -> TuplePk<E> {
    let cols: Vec<_> = <E::PrimaryKey as Iterable>::iter()
        .map(|pk| pk.into_column())
        .collect();
    TuplePk(cols)
}

impl<E: EntityTrait> ModelKey<E> for UnitPk<E> {
    type Type = Value;
    fn get(&self, model: &E::Model) -> Self::Type {
        model.get(self.0)
    }
}

impl<E: EntityTrait> ModelKey<E> for PairPk<E> {
    type Type = (Value, Value);
    fn get(&self, model: &E::Model) -> Self::Type {
        (model.get(self.0), model.get(self.1))
    }
}

impl<E: EntityTrait> ModelKey<E> for TuplePk<E> {
    type Type = Vec<Value>;
    fn get(&self, model: &E::Model) -> Self::Type {
        let mut key = Vec::new();
        for col in self.0.iter() {
            key.push(model.get(*col));
        }
        key
    }
}

fn consolidate_query_result_of<L, R, KEY: ModelKey<L>>(
    mut rows: Vec<(L::Model, Option<R::Model>)>,
    model_key: KEY,
) -> Vec<(L::Model, Vec<R::Model>)>
where
    L: EntityTrait,
    R: EntityTrait,
{
    // group items by unique key on left model
    let mut hashmap: HashMap<KEY::Type, Vec<R::Model>> =
        rows.iter_mut().fold(HashMap::new(), |mut acc, row| {
            let key = model_key.get(&row.0); // keep left model in place
            if let Some(value) = row.1.take() {
                // take ownership of right model
                if let Some(vec) = acc.get_mut(&key) {
                    vec.push(value)
                } else {
                    acc.insert(key, vec![value]);
                }
            } else {
                acc.entry(key).or_default(); // insert empty vec
            }

            acc
        });

    // re-iterate so that we keep the same order
    rows.into_iter()
        .filter_map(|(l_model, _)| {
            // right model is empty here already
            let l_pk = model_key.get(&l_model);
            // the first time we encounter a left model, the value is taken
            // subsequently the key will be empty
            let r_models = hashmap.remove(&l_pk);
            r_models.map(|r_models| (l_model, r_models))
        })
        .collect()
}

// this consolidate query result of a T topology
// where L -> M and L -> R
fn consolidate_query_result_of_tee<L, M, R, KEY: ModelKey<L>>(
    mut rows: Vec<(L::Model, Option<M::Model>, Option<R::Model>)>,
    model_key: KEY,
) -> Vec<(L::Model, Vec<M::Model>, Vec<R::Model>)>
where
    L: EntityTrait,
    M: EntityTrait,
    R: EntityTrait,
{
    struct Slot<M, R> {
        m: Vec<M>,
        r: Vec<R>,
    }

    impl<M, R> Default for Slot<M, R> {
        fn default() -> Self {
            Self {
                m: vec![],
                r: vec![],
            }
        }
    }

    // group items by unique key on left model
    let mut hashmap: HashMap<KEY::Type, Slot<M::Model, R::Model>> =
        rows.iter_mut().fold(HashMap::new(), |mut acc, row| {
            let key = model_key.get(&row.0);
            match (row.1.take(), row.2.take()) {
                (Some(m), Some(r)) => {
                    if let Some(slot) = acc.get_mut(&key) {
                        slot.m.push(m);
                        slot.r.push(r);
                    } else {
                        acc.insert(
                            key,
                            Slot {
                                m: vec![m],
                                r: vec![r],
                            },
                        );
                    }
                }
                (Some(m), None) => {
                    if let Some(slot) = acc.get_mut(&key) {
                        slot.m.push(m);
                    } else {
                        acc.insert(
                            key,
                            Slot {
                                m: vec![m],
                                r: vec![],
                            },
                        );
                    }
                }
                (None, Some(r)) => {
                    if let Some(slot) = acc.get_mut(&key) {
                        slot.r.push(r);
                    } else {
                        acc.insert(
                            key,
                            Slot {
                                m: vec![],
                                r: vec![r],
                            },
                        );
                    }
                }
                (None, None) => {
                    acc.entry(key).or_default(); // insert empty vec
                }
            }

            acc
        });

    // re-iterate so that we keep the same order
    rows.into_iter()
        .filter_map(|(l_model, _, _)| {
            let l_pk = model_key.get(&l_model);
            let mr_models = hashmap.remove(&l_pk);
            // if both L -> M and L -> R is one to many
            // it's possible for them to have duplicates
            mr_models.map(|Slot { m, r }| {
                (
                    l_model,
                    retain_unique_models::<M>(m),
                    retain_unique_models::<R>(r),
                )
            })
        })
        .collect()
}

// this consolidate query result of a chained topology
// where L -> M and M -> R
fn consolidate_query_result_of_chain<L, M, R, KEY: ModelKey<L>>(
    mut rows: Vec<(L::Model, Option<M::Model>, Option<R::Model>)>,
    model_key: KEY,
) -> Vec<(L::Model, Vec<(M::Model, Vec<R::Model>)>)>
where
    L: EntityTrait,
    M: EntityTrait,
    R: EntityTrait,
{
    // group items by unique key on left model
    let mut hashmap: HashMap<KEY::Type, Vec<(M::Model, Option<R::Model>)>> =
        rows.iter_mut().fold(HashMap::new(), |mut acc, row| {
            let key = model_key.get(&row.0);
            match (row.1.take(), row.2.take()) {
                (Some(m), Some(r)) => {
                    if let Some(slot) = acc.get_mut(&key) {
                        slot.push((m, Some(r)));
                    } else {
                        acc.insert(key, vec![(m, Some(r))]);
                    }
                }
                (Some(m), None) => {
                    if let Some(slot) = acc.get_mut(&key) {
                        slot.push((m, None));
                    } else {
                        acc.insert(key, vec![(m, None)]);
                    }
                }
                (None, Some(_)) => {
                    panic!(
                        "Impossible to have R when M ({}) -> R ({})",
                        M::default().as_str(),
                        R::default().as_str()
                    )
                }
                (None, None) => {
                    acc.entry(key).or_default(); // insert empty vec
                }
            }

            acc
        });

    // re-iterate so that we keep the same order
    rows.into_iter()
        .filter_map(|(l_model, _, _)| {
            let l_pk = model_key.get(&l_model);
            let mr_models = hashmap.remove(&l_pk);
            mr_models.map(|mr_models| (l_model, consolidate_query_result::<M, R>(mr_models)))
        })
        .collect()
}

fn retain_unique_models_of<L, KEY: ModelKey<L>>(
    mut rows: Vec<L::Model>,
    model_key: KEY,
) -> Vec<L::Model>
where
    L: EntityTrait,
{
    let mut seen = HashSet::new();

    rows.retain(|model| seen.insert(model_key.get(model)));

    rows
}

/// This is the legacy consolidate algorithm. Kept for reference
#[allow(dead_code)]
fn consolidate_query_result_of_ordered_rows<L, R>(
    rows: Vec<(L::Model, Option<R::Model>)>,
) -> Vec<(L::Model, Vec<R::Model>)>
where
    L: EntityTrait,
    R: EntityTrait,
{
    let mut acc: Vec<(L::Model, Vec<R::Model>)> = Vec::new();
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
            if same_l {
                if let Some(r) = r {
                    last_r.push(r);
                    continue;
                }
            }
        }
        let rows = match r {
            Some(r) => vec![r],
            None => vec![],
        };
        acc.push((l, rows));
    }
    acc
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    fn cake_model(id: i32) -> crate::tests_cfg::cake::Model {
        let name = match id {
            1 => "apple cake",
            2 => "orange cake",
            3 => "fruit cake",
            4 => "chocolate cake",
            _ => "",
        }
        .to_string();
        crate::tests_cfg::cake::Model { id, name }
    }

    fn filling_model(id: i32) -> crate::tests_cfg::filling::Model {
        let name = match id {
            1 => "apple sauce",
            2 => "orange jam",
            3 => "fruit salad",
            4 => "chocolate chips",
            _ => "",
        }
        .to_string();
        crate::tests_cfg::filling::Model {
            id,
            name,
            vendor_id: Some(1),
            ignored_attr: 0,
        }
    }

    fn fruit_model(id: i32) -> crate::tests_cfg::fruit::Model {
        fruit_model_for(id, None)
    }

    fn fruit_model_for(id: i32, cake_id: Option<i32>) -> crate::tests_cfg::fruit::Model {
        let name = match id {
            1 => "apple",
            2 => "orange",
            3 => "grape",
            4 => "strawberry",
            _ => "",
        }
        .to_string();
        crate::tests_cfg::fruit::Model { id, name, cake_id }
    }

    fn vendor_model(id: i32) -> crate::tests_cfg::vendor::Model {
        let name = match id {
            1 => "Apollo",
            2 => "Benny",
            3 => "Christine",
            4 => "David",
            _ => "",
        }
        .to_string();
        crate::tests_cfg::vendor::Model { id, name }
    }

    fn cake_with_fruit(
        cake_id: i32,
        fruit_id: i32,
    ) -> (
        crate::tests_cfg::cake::Model,
        crate::tests_cfg::fruit::Model,
    ) {
        (
            cake_model(cake_id),
            fruit_model_for(fruit_id, Some(cake_id)),
        )
    }

    fn cake_and_filling(
        cake_id: i32,
        filling_id: i32,
    ) -> (
        crate::tests_cfg::cake::Model,
        crate::tests_cfg::filling::Model,
    ) {
        (cake_model(cake_id), filling_model(filling_id))
    }

    fn cake_and_vendor(
        cake_id: i32,
        vendor_id: i32,
    ) -> (
        crate::tests_cfg::cake::Model,
        crate::tests_cfg::vendor::Model,
    ) {
        (cake_model(cake_id), vendor_model(vendor_id))
    }

    #[smol_potat::test]
    async fn also_related() -> Result<(), crate::DbErr> {
        use crate::tests_cfg::*;
        use crate::{DbBackend, EntityTrait, MockDatabase, Statement, Transaction};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[cake_with_fruit(1, 1)]])
            .into_connection();

        assert_eq!(
            Cake::find().find_also_related(Fruit).all(&db).await?,
            [(cake_model(1), Some(fruit_model_for(1, Some(1))))]
        );

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::many([Statement::from_sql_and_values(
                DbBackend::Postgres,
                [
                    r#"SELECT "cake"."id" AS "A_id", "cake"."name" AS "A_name","#,
                    r#""fruit"."id" AS "B_id", "fruit"."name" AS "B_name", "fruit"."cake_id" AS "B_cake_id""#,
                    r#"FROM "cake""#,
                    r#"LEFT JOIN "fruit" ON "cake"."id" = "fruit"."cake_id""#,
                ]
                .join(" ")
                .as_str(),
                []
            ),])]
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn also_related_2() -> Result<(), crate::DbErr> {
        use crate::tests_cfg::*;
        use crate::{DbBackend, EntityTrait, MockDatabase};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[cake_with_fruit(1, 1), cake_with_fruit(1, 2)]])
            .into_connection();

        assert_eq!(
            Cake::find().find_also_related(Fruit).all(&db).await?,
            [
                (cake_model(1), Some(fruit_model_for(1, Some(1)))),
                (cake_model(1), Some(fruit_model_for(2, Some(1))))
            ]
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn also_related_3() -> Result<(), crate::DbErr> {
        use crate::tests_cfg::*;
        use crate::{DbBackend, EntityTrait, MockDatabase};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[
                cake_with_fruit(1, 1),
                cake_with_fruit(1, 2),
                cake_with_fruit(2, 3),
            ]])
            .into_connection();

        assert_eq!(
            Cake::find().find_also_related(Fruit).all(&db).await?,
            [
                (cake_model(1), Some(fruit_model_for(1, Some(1)))),
                (cake_model(1), Some(fruit_model_for(2, Some(1)))),
                (cake_model(2), Some(fruit_model_for(3, Some(2))))
            ]
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn also_related_4() -> Result<(), crate::DbErr> {
        use crate::tests_cfg::*;
        use crate::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[
                cake_with_fruit(1, 1).into_mock_row(),
                cake_with_fruit(1, 2).into_mock_row(),
                cake_with_fruit(2, 3).into_mock_row(),
                (cake_model(3), None::<fruit::Model>).into_mock_row(),
            ]])
            .into_connection();

        assert_eq!(
            Cake::find().find_also_related(Fruit).all(&db).await?,
            [
                (cake_model(1), Some(fruit_model_for(1, Some(1)))),
                (cake_model(1), Some(fruit_model_for(2, Some(1)))),
                (cake_model(2), Some(fruit_model_for(3, Some(2)))),
                (cake_model(3), None)
            ]
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn also_related_many_to_many() -> Result<(), crate::DbErr> {
        use crate::tests_cfg::*;
        use crate::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[
                cake_and_filling(1, 1).into_mock_row(),
                cake_and_filling(1, 2).into_mock_row(),
                cake_and_filling(2, 2).into_mock_row(),
            ]])
            .into_connection();

        assert_eq!(
            Cake::find().find_also_related(Filling).all(&db).await?,
            [
                (cake_model(1), Some(filling_model(1))),
                (cake_model(1), Some(filling_model(2))),
                (cake_model(2), Some(filling_model(2))),
            ]
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn also_related_many_to_many_2() -> Result<(), crate::DbErr> {
        use crate::tests_cfg::*;
        use crate::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[
                cake_and_filling(1, 1).into_mock_row(),
                cake_and_filling(1, 2).into_mock_row(),
                cake_and_filling(2, 2).into_mock_row(),
                (cake_model(3), None::<filling::Model>).into_mock_row(),
            ]])
            .into_connection();

        assert_eq!(
            Cake::find().find_also_related(Filling).all(&db).await?,
            [
                (cake_model(1), Some(filling_model(1))),
                (cake_model(1), Some(filling_model(2))),
                (cake_model(2), Some(filling_model(2))),
                (cake_model(3), None)
            ]
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn with_related() -> Result<(), crate::DbErr> {
        use crate::tests_cfg::*;
        use crate::{DbBackend, EntityTrait, MockDatabase, Statement, Transaction};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[
                cake_with_fruit(1, 1),
                cake_with_fruit(2, 2),
                cake_with_fruit(2, 3),
            ]])
            .into_connection();

        assert_eq!(
            Cake::find().find_with_related(Fruit).all(&db).await?,
            [
                (cake_model(1), vec![fruit_model_for(1, Some(1))]),
                (
                    cake_model(2),
                    vec![fruit_model_for(2, Some(2)), fruit_model_for(3, Some(2))]
                )
            ]
        );

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::many([Statement::from_sql_and_values(
                DbBackend::Postgres,
                [
                    r#"SELECT "cake"."id" AS "A_id", "cake"."name" AS "A_name","#,
                    r#""fruit"."id" AS "B_id", "fruit"."name" AS "B_name", "fruit"."cake_id" AS "B_cake_id""#,
                    r#"FROM "cake""#,
                    r#"LEFT JOIN "fruit" ON "cake"."id" = "fruit"."cake_id""#,
                    r#"ORDER BY "cake"."id" ASC"#
                ]
                .join(" ")
                .as_str(),
                []
            ),])]
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn with_related_2() -> Result<(), crate::DbErr> {
        use crate::tests_cfg::*;
        use crate::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[
                cake_with_fruit(1, 1).into_mock_row(),
                cake_with_fruit(2, 2).into_mock_row(),
                cake_with_fruit(2, 3).into_mock_row(),
                cake_with_fruit(2, 4).into_mock_row(),
            ]])
            .into_connection();

        assert_eq!(
            Cake::find().find_with_related(Fruit).all(&db).await?,
            [
                (cake_model(1), vec![fruit_model_for(1, Some(1)),]),
                (
                    cake_model(2),
                    vec![
                        fruit_model_for(2, Some(2)),
                        fruit_model_for(3, Some(2)),
                        fruit_model_for(4, Some(2)),
                    ]
                ),
            ]
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn with_related_empty() -> Result<(), crate::DbErr> {
        use crate::tests_cfg::*;
        use crate::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[
                cake_with_fruit(1, 1).into_mock_row(),
                cake_with_fruit(2, 2).into_mock_row(),
                cake_with_fruit(2, 3).into_mock_row(),
                cake_with_fruit(2, 4).into_mock_row(),
                (cake_model(3), None::<fruit::Model>).into_mock_row(),
            ]])
            .into_connection();

        assert_eq!(
            Cake::find().find_with_related(Fruit).all(&db).await?,
            [
                (cake_model(1), vec![fruit_model_for(1, Some(1)),]),
                (
                    cake_model(2),
                    vec![
                        fruit_model_for(2, Some(2)),
                        fruit_model_for(3, Some(2)),
                        fruit_model_for(4, Some(2)),
                    ]
                ),
                (cake_model(3), vec![])
            ]
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn with_related_many_to_many() -> Result<(), crate::DbErr> {
        use crate::tests_cfg::*;
        use crate::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[
                cake_and_filling(1, 1).into_mock_row(),
                cake_and_filling(1, 2).into_mock_row(),
                cake_and_filling(2, 2).into_mock_row(),
            ]])
            .into_connection();

        assert_eq!(
            Cake::find().find_with_related(Filling).all(&db).await?,
            [
                (cake_model(1), vec![filling_model(1), filling_model(2)]),
                (cake_model(2), vec![filling_model(2)]),
            ]
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn with_related_many_to_many_2() -> Result<(), crate::DbErr> {
        use crate::tests_cfg::*;
        use crate::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[
                cake_and_filling(1, 1).into_mock_row(),
                cake_and_filling(1, 2).into_mock_row(),
                cake_and_filling(2, 2).into_mock_row(),
                (cake_model(3), None::<filling::Model>).into_mock_row(),
            ]])
            .into_connection();

        assert_eq!(
            Cake::find().find_with_related(Filling).all(&db).await?,
            [
                (cake_model(1), vec![filling_model(1), filling_model(2)]),
                (cake_model(2), vec![filling_model(2)]),
                (cake_model(3), vec![])
            ]
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn also_linked_base() -> Result<(), crate::DbErr> {
        use crate::tests_cfg::*;
        use crate::{DbBackend, EntityTrait, MockDatabase, Statement, Transaction};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[cake_and_vendor(1, 1)]])
            .into_connection();

        assert_eq!(
            Cake::find()
                .find_also_linked(entity_linked::CakeToFillingVendor)
                .all(&db)
                .await?,
            [(cake_model(1), Some(vendor_model(1)))]
        );

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::many([Statement::from_sql_and_values(
                DbBackend::Postgres,
                [
                    r#"SELECT "cake"."id" AS "A_id", "cake"."name" AS "A_name","#,
                    r#""r2"."id" AS "B_id", "r2"."name" AS "B_name""#,
                    r#"FROM "cake""#,
                    r#"LEFT JOIN "cake_filling" AS "r0" ON "cake"."id" = "r0"."cake_id""#,
                    r#"LEFT JOIN "filling" AS "r1" ON "r0"."filling_id" = "r1"."id""#,
                    r#"LEFT JOIN "vendor" AS "r2" ON "r1"."vendor_id" = "r2"."id""#,
                ]
                .join(" ")
                .as_str(),
                []
            ),])]
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn also_linked_same_cake() -> Result<(), crate::DbErr> {
        use crate::tests_cfg::*;
        use crate::{DbBackend, EntityTrait, MockDatabase};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[
                cake_and_vendor(1, 1),
                cake_and_vendor(1, 2),
                cake_and_vendor(2, 3),
            ]])
            .into_connection();

        assert_eq!(
            Cake::find()
                .find_also_linked(entity_linked::CakeToFillingVendor)
                .all(&db)
                .await?,
            [
                (cake_model(1), Some(vendor_model(1))),
                (cake_model(1), Some(vendor_model(2))),
                (cake_model(2), Some(vendor_model(3)))
            ]
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn also_linked_same_vendor() -> Result<(), crate::DbErr> {
        use crate::tests_cfg::*;
        use crate::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[
                cake_and_vendor(1, 1).into_mock_row(),
                cake_and_vendor(2, 1).into_mock_row(),
                cake_and_vendor(3, 2).into_mock_row(),
            ]])
            .into_connection();

        assert_eq!(
            Cake::find()
                .find_also_linked(entity_linked::CakeToFillingVendor)
                .all(&db)
                .await?,
            [
                (cake_model(1), Some(vendor_model(1))),
                (cake_model(2), Some(vendor_model(1))),
                (cake_model(3), Some(vendor_model(2))),
            ]
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn also_linked_many_to_many() -> Result<(), crate::DbErr> {
        use crate::tests_cfg::*;
        use crate::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[
                cake_and_vendor(1, 1).into_mock_row(),
                cake_and_vendor(1, 2).into_mock_row(),
                cake_and_vendor(1, 3).into_mock_row(),
                cake_and_vendor(2, 1).into_mock_row(),
                cake_and_vendor(2, 2).into_mock_row(),
            ]])
            .into_connection();

        assert_eq!(
            Cake::find()
                .find_also_linked(entity_linked::CakeToFillingVendor)
                .all(&db)
                .await?,
            [
                (cake_model(1), Some(vendor_model(1))),
                (cake_model(1), Some(vendor_model(2))),
                (cake_model(1), Some(vendor_model(3))),
                (cake_model(2), Some(vendor_model(1))),
                (cake_model(2), Some(vendor_model(2))),
            ]
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn also_linked_empty() -> Result<(), crate::DbErr> {
        use crate::tests_cfg::*;
        use crate::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[
                cake_and_vendor(1, 1).into_mock_row(),
                cake_and_vendor(2, 2).into_mock_row(),
                cake_and_vendor(3, 3).into_mock_row(),
                (cake_model(4), None::<vendor::Model>).into_mock_row(),
            ]])
            .into_connection();

        assert_eq!(
            Cake::find()
                .find_also_linked(entity_linked::CakeToFillingVendor)
                .all(&db)
                .await?,
            [
                (cake_model(1), Some(vendor_model(1))),
                (cake_model(2), Some(vendor_model(2))),
                (cake_model(3), Some(vendor_model(3))),
                (cake_model(4), None)
            ]
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn with_linked_base() -> Result<(), crate::DbErr> {
        use crate::tests_cfg::*;
        use crate::{DbBackend, EntityTrait, MockDatabase, Statement, Transaction};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[
                cake_and_vendor(1, 1),
                cake_and_vendor(2, 2),
                cake_and_vendor(2, 3),
            ]])
            .into_connection();

        assert_eq!(
            Cake::find()
                .find_with_linked(entity_linked::CakeToFillingVendor)
                .all(&db)
                .await?,
            [
                (cake_model(1), vec![vendor_model(1)]),
                (cake_model(2), vec![vendor_model(2), vendor_model(3)])
            ]
        );

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::many([Statement::from_sql_and_values(
                DbBackend::Postgres,
                [
                    r#"SELECT "cake"."id" AS "A_id", "cake"."name" AS "A_name","#,
                    r#""r2"."id" AS "B_id", "r2"."name" AS "B_name" FROM "cake""#,
                    r#"LEFT JOIN "cake_filling" AS "r0" ON "cake"."id" = "r0"."cake_id""#,
                    r#"LEFT JOIN "filling" AS "r1" ON "r0"."filling_id" = "r1"."id""#,
                    r#"LEFT JOIN "vendor" AS "r2" ON "r1"."vendor_id" = "r2"."id""#,
                ]
                .join(" ")
                .as_str(),
                []
            ),])]
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn with_linked_same_vendor() -> Result<(), crate::DbErr> {
        use crate::tests_cfg::*;
        use crate::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[
                cake_and_vendor(1, 1).into_mock_row(),
                cake_and_vendor(2, 2).into_mock_row(),
                cake_and_vendor(3, 2).into_mock_row(),
            ]])
            .into_connection();

        assert_eq!(
            Cake::find()
                .find_with_linked(entity_linked::CakeToFillingVendor)
                .all(&db)
                .await?,
            [
                (cake_model(1), vec![vendor_model(1)]),
                (cake_model(2), vec![vendor_model(2)]),
                (cake_model(3), vec![vendor_model(2)])
            ]
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn with_linked_empty() -> Result<(), crate::DbErr> {
        use crate::tests_cfg::*;
        use crate::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[
                cake_and_vendor(1, 1).into_mock_row(),
                cake_and_vendor(2, 1).into_mock_row(),
                cake_and_vendor(2, 2).into_mock_row(),
                (cake_model(3), None::<vendor::Model>).into_mock_row(),
            ]])
            .into_connection();

        assert_eq!(
            Cake::find()
                .find_with_linked(entity_linked::CakeToFillingVendor)
                .all(&db)
                .await?,
            [
                (cake_model(1), vec![vendor_model(1)]),
                (cake_model(2), vec![vendor_model(1), vendor_model(2)]),
                (cake_model(3), vec![])
            ]
        );

        Ok(())
    }

    // normally would not happen
    #[smol_potat::test]
    async fn with_linked_repeated() -> Result<(), crate::DbErr> {
        use crate::tests_cfg::*;
        use crate::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[
                cake_and_vendor(1, 1).into_mock_row(),
                cake_and_vendor(1, 1).into_mock_row(),
                cake_and_vendor(2, 1).into_mock_row(),
                cake_and_vendor(2, 2).into_mock_row(),
            ]])
            .into_connection();

        assert_eq!(
            Cake::find()
                .find_with_linked(entity_linked::CakeToFillingVendor)
                .all(&db)
                .await?,
            [
                (cake_model(1), vec![vendor_model(1), vendor_model(1)]),
                (cake_model(2), vec![vendor_model(1), vendor_model(2)]),
            ]
        );

        Ok(())
    }

    #[test]
    fn test_retain_unique_models() {
        use crate::tests_cfg::Cake;
        assert_eq!(
            super::retain_unique_models::<Cake>(vec![
                cake_model(1),
                cake_model(1),
                cake_model(2),
                cake_model(2),
                cake_model(3),
            ]),
            [cake_model(1), cake_model(2), cake_model(3)]
        );
    }

    #[test]
    #[rustfmt::skip]
    fn test_consolidate_tee() {
        use crate::tests_cfg::{Cake, Filling, Fruit};

        assert_eq!(
            super::consolidate_query_result_tee::<Cake, Fruit, Filling>(vec![
                (cake_model(1), Some(fruit_model(1)), Some(filling_model(1))),
            ]),
            vec![(cake_model(1), vec![fruit_model(1)], vec![filling_model(1)])]
        );

        assert_eq!(
            super::consolidate_query_result_tee::<Cake, Fruit, Filling>(vec![
                (cake_model(1), Some(fruit_model(1)), None),
            ]),
            vec![(cake_model(1), vec![fruit_model(1)], vec![])]
        );

        assert_eq!(
            super::consolidate_query_result_tee::<Cake, Fruit, Filling>(vec![
                (cake_model(1), None, Some(filling_model(1))),
            ]),
            vec![(cake_model(1), vec![], vec![filling_model(1)])]
        );

        assert_eq!(
            super::consolidate_query_result_tee::<Cake, Fruit, Filling>(vec![
                (cake_model(1), Some(fruit_model(1)), Some(filling_model(1))),
                (cake_model(1), Some(fruit_model(1)), Some(filling_model(2))),
            ]),
            vec![(
                cake_model(1),
                vec![fruit_model(1)],
                vec![filling_model(1), filling_model(2)]
            )]
        );

        assert_eq!(
            super::consolidate_query_result_tee::<Cake, Fruit, Filling>(vec![
                (cake_model(1), Some(fruit_model(1)), Some(filling_model(1))),
                (cake_model(1), Some(fruit_model(1)), Some(filling_model(2))),
                (cake_model(1), Some(fruit_model(1)), Some(filling_model(3))),
                (cake_model(1), Some(fruit_model(2)), Some(filling_model(1))),
                (cake_model(1), Some(fruit_model(2)), Some(filling_model(2))),
                (cake_model(1), Some(fruit_model(2)), Some(filling_model(3))),
            ]),
            vec![(
                cake_model(1),
                vec![fruit_model(1), fruit_model(2)],
                vec![filling_model(1), filling_model(2), filling_model(3)]
            )]
        );

        assert_eq!(
            super::consolidate_query_result_tee::<Cake, Fruit, Filling>(vec![
                (cake_model(1), Some(fruit_model(1)), Some(filling_model(1))),
                (cake_model(1), Some(fruit_model(1)), Some(filling_model(2))),
                (cake_model(1), Some(fruit_model(1)), Some(filling_model(3))),
                (cake_model(1), Some(fruit_model(2)), Some(filling_model(1))),
                (cake_model(1), Some(fruit_model(2)), Some(filling_model(2))),
                (cake_model(1), Some(fruit_model(2)), Some(filling_model(3))),
                (cake_model(2), Some(fruit_model(1)), Some(filling_model(2))),
                (cake_model(2), Some(fruit_model(2)), Some(filling_model(2))),
                (cake_model(3), Some(fruit_model(3)), None),
                (cake_model(4), None, None),
            ]),
            vec![(
                cake_model(1),
                vec![fruit_model(1), fruit_model(2)],
                vec![filling_model(1), filling_model(2), filling_model(3)]
            ), (
                cake_model(2),
                vec![fruit_model(1), fruit_model(2)],
                vec![filling_model(2)]
            ), (
                cake_model(3),
                vec![fruit_model(3)],
                vec![]
            ), (
                cake_model(4),
                vec![],
                vec![]
            )]
        );
    }

    #[test]
    #[rustfmt::skip]
    fn test_consolidate_chain() {
        use crate::tests_cfg::{Cake, Filling, Fruit};

        assert_eq!(
            super::consolidate_query_result_chain::<Cake, Fruit, Filling>(vec![
                (cake_model(1), Some(fruit_model(1)), Some(filling_model(1))),
            ]),
            vec![(cake_model(1), vec![(fruit_model(1), vec![filling_model(1)])])]
        );

        assert_eq!(
            super::consolidate_query_result_chain::<Cake, Fruit, Filling>(vec![
                (cake_model(1), Some(fruit_model(1)), None),
            ]),
            vec![(cake_model(1), vec![(fruit_model(1), vec![])])]
        );

        assert_eq!(
            super::consolidate_query_result_chain::<Cake, Fruit, Filling>(vec![
                (cake_model(1), None, None),
            ]),
            vec![(cake_model(1), vec![])]
        );

        assert_eq!(
            super::consolidate_query_result_chain::<Cake, Fruit, Filling>(vec![
                (cake_model(1), Some(fruit_model(1)), Some(filling_model(1))),
                (cake_model(1), Some(fruit_model(1)), Some(filling_model(2))),
            ]),
            vec![(
                cake_model(1),
                vec![(fruit_model(1), vec![filling_model(1), filling_model(2)])]
            )]
        );

        assert_eq!(
            super::consolidate_query_result_chain::<Cake, Fruit, Filling>(vec![
                (cake_model(1), Some(fruit_model(1)), Some(filling_model(1))),
                (cake_model(1), Some(fruit_model(1)), Some(filling_model(2))),
                (cake_model(1), Some(fruit_model(1)), Some(filling_model(3))),
            ]),
            vec![(
                cake_model(1),
                vec![(fruit_model(1), vec![filling_model(1), filling_model(2), filling_model(3)])]
            )]
        );

        assert_eq!(
            super::consolidate_query_result_chain::<Cake, Fruit, Filling>(vec![
                (cake_model(1), Some(fruit_model(1)), Some(filling_model(1))),
                (cake_model(1), Some(fruit_model(1)), Some(filling_model(2))),
                (cake_model(1), Some(fruit_model(1)), Some(filling_model(3))),
                (cake_model(1), Some(fruit_model(2)), Some(filling_model(2))),
                (cake_model(1), Some(fruit_model(2)), Some(filling_model(3))),
            ]),
            vec![(
                cake_model(1),
                vec![
                    (fruit_model(1), vec![filling_model(1), filling_model(2), filling_model(3)]),
                    (fruit_model(2), vec![filling_model(2), filling_model(3)]),
                ]
            )]
        );

        assert_eq!(
            super::consolidate_query_result_chain::<Cake, Fruit, Filling>(vec![
                (cake_model(1), Some(fruit_model(1)), Some(filling_model(1))),
                (cake_model(1), Some(fruit_model(1)), Some(filling_model(2))),
                (cake_model(1), Some(fruit_model(1)), Some(filling_model(3))),
                (cake_model(1), Some(fruit_model(2)), Some(filling_model(2))),
                (cake_model(1), Some(fruit_model(2)), Some(filling_model(3))),
                (cake_model(2), Some(fruit_model(3)), Some(filling_model(4))),
            ]),
            vec![(
                cake_model(1),
                vec![
                    (fruit_model(1), vec![filling_model(1), filling_model(2), filling_model(3)]),
                    (fruit_model(2), vec![filling_model(2), filling_model(3)]),
                ]
            ), (
                cake_model(2),
                vec![(fruit_model(3), vec![filling_model(4)])]
            )]
        );

        assert_eq!(
            super::consolidate_query_result_chain::<Cake, Fruit, Filling>(vec![
                (cake_model(1), Some(fruit_model(1)), Some(filling_model(1))),
                (cake_model(1), Some(fruit_model(1)), Some(filling_model(2))),
                (cake_model(1), Some(fruit_model(1)), Some(filling_model(3))),
                (cake_model(1), Some(fruit_model(2)), Some(filling_model(2))),
                (cake_model(1), Some(fruit_model(2)), Some(filling_model(3))),
                (cake_model(2), Some(fruit_model(3)), Some(filling_model(3))),
                (cake_model(2), Some(fruit_model(3)), Some(filling_model(4))),
                (cake_model(3), Some(fruit_model(4)), None),
                (cake_model(4), None, None),
            ]),
            vec![(
                cake_model(1),
                vec![
                    (fruit_model(1), vec![filling_model(1), filling_model(2), filling_model(3)]),
                    (fruit_model(2), vec![filling_model(2), filling_model(3)]),
                ]
            ), (
                cake_model(2),
                vec![(fruit_model(3), vec![filling_model(3), filling_model(4)])]
            ), (
                cake_model(3),
                vec![(fruit_model(4), vec![])]
            ), (
                cake_model(4),
                vec![]
            )]
        );
    }
}
