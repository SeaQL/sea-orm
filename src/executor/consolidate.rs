use crate::{
    EntityTrait, Iterable, ModelTrait, PrimaryKeyArity, PrimaryKeyToColumn, PrimaryKeyTrait,
};
use sea_query::Value;
use std::collections::HashMap;
use std::hash::Hash;

#[allow(clippy::unwrap_used)]
pub(super) fn consolidate_query_result<L, R>(
    rows: Vec<(L::Model, Option<R::Model>)>,
) -> Vec<(L::Model, Vec<R::Model>)>
where
    L: EntityTrait,
    R: EntityTrait,
{
    match <<L::PrimaryKey as PrimaryKeyTrait>::ValueType as PrimaryKeyArity>::ARITY {
        1 => {
            let col = <L::PrimaryKey as Iterable>::iter()
                .next()
                .unwrap()
                .into_column();
            consolidate_query_result_of::<L, R, UnitPk<L>>(rows, UnitPk(col))
        }
        2 => {
            let mut iter = <L::PrimaryKey as Iterable>::iter();
            let col1 = iter.next().unwrap().into_column();
            let col2 = iter.next().unwrap().into_column();
            consolidate_query_result_of::<L, R, PairPk<L>>(rows, PairPk(col1, col2))
        }
        _ => {
            let cols: Vec<_> = <L::PrimaryKey as Iterable>::iter()
                .map(|pk| pk.into_column())
                .collect();
            consolidate_query_result_of::<L, R, TuplePk<L>>(rows, TuplePk(cols))
        }
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
                let vec: Option<&mut Vec<R::Model>> = acc.get_mut(&key);
                if let Some(vec) = vec {
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

    fn filling_model(id: i32) -> sea_orm::tests_cfg::filling::Model {
        let name = match id {
            1 => "apple sauce",
            2 => "orange jam",
            3 => "fruit salad",
            4 => "chocolate chips",
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

    fn vendor_model(id: i32) -> sea_orm::tests_cfg::vendor::Model {
        let name = match id {
            1 => "Apollo",
            2 => "Benny",
            3 => "Christine",
            4 => "David",
            _ => "",
        }
        .to_string();
        sea_orm::tests_cfg::vendor::Model { id, name }
    }

    fn cake_with_fruit(
        cake_id: i32,
        fruit_id: i32,
    ) -> (
        sea_orm::tests_cfg::cake::Model,
        sea_orm::tests_cfg::fruit::Model,
    ) {
        (cake_model(cake_id), fruit_model(fruit_id, Some(cake_id)))
    }

    fn cake_and_filling(
        cake_id: i32,
        filling_id: i32,
    ) -> (
        sea_orm::tests_cfg::cake::Model,
        sea_orm::tests_cfg::filling::Model,
    ) {
        (cake_model(cake_id), filling_model(filling_id))
    }

    fn cake_and_vendor(
        cake_id: i32,
        vendor_id: i32,
    ) -> (
        sea_orm::tests_cfg::cake::Model,
        sea_orm::tests_cfg::vendor::Model,
    ) {
        (cake_model(cake_id), vendor_model(vendor_id))
    }

    #[smol_potat::test]
    pub async fn also_related() -> Result<(), sea_orm::DbErr> {
        use sea_orm::tests_cfg::*;
        use sea_orm::{DbBackend, EntityTrait, MockDatabase, Statement, Transaction};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[cake_with_fruit(1, 1)]])
            .into_connection();

        assert_eq!(
            Cake::find().find_also_related(Fruit).all(&db).await?,
            [(cake_model(1), Some(fruit_model(1, Some(1))))]
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
    pub async fn also_related_2() -> Result<(), sea_orm::DbErr> {
        use sea_orm::tests_cfg::*;
        use sea_orm::{DbBackend, EntityTrait, MockDatabase};

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[cake_with_fruit(1, 1), cake_with_fruit(1, 2)]])
            .into_connection();

        assert_eq!(
            Cake::find().find_also_related(Fruit).all(&db).await?,
            [
                (cake_model(1), Some(fruit_model(1, Some(1)))),
                (cake_model(1), Some(fruit_model(2, Some(1))))
            ]
        );

        Ok(())
    }

    #[smol_potat::test]
    pub async fn also_related_3() -> Result<(), sea_orm::DbErr> {
        use sea_orm::tests_cfg::*;
        use sea_orm::{DbBackend, EntityTrait, MockDatabase};

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
                (cake_model(1), Some(fruit_model(1, Some(1)))),
                (cake_model(1), Some(fruit_model(2, Some(1)))),
                (cake_model(2), Some(fruit_model(3, Some(2))))
            ]
        );

        Ok(())
    }

    #[smol_potat::test]
    pub async fn also_related_4() -> Result<(), sea_orm::DbErr> {
        use sea_orm::tests_cfg::*;
        use sea_orm::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

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
                (cake_model(1), Some(fruit_model(1, Some(1)))),
                (cake_model(1), Some(fruit_model(2, Some(1)))),
                (cake_model(2), Some(fruit_model(3, Some(2)))),
                (cake_model(3), None)
            ]
        );

        Ok(())
    }

    #[smol_potat::test]
    pub async fn also_related_many_to_many() -> Result<(), sea_orm::DbErr> {
        use sea_orm::tests_cfg::*;
        use sea_orm::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

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
    pub async fn also_related_many_to_many_2() -> Result<(), sea_orm::DbErr> {
        use sea_orm::tests_cfg::*;
        use sea_orm::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

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
    pub async fn with_related() -> Result<(), sea_orm::DbErr> {
        use sea_orm::tests_cfg::*;
        use sea_orm::{DbBackend, EntityTrait, MockDatabase, Statement, Transaction};

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
                (cake_model(1), vec![fruit_model(1, Some(1))]),
                (
                    cake_model(2),
                    vec![fruit_model(2, Some(2)), fruit_model(3, Some(2))]
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
    pub async fn with_related_2() -> Result<(), sea_orm::DbErr> {
        use sea_orm::tests_cfg::*;
        use sea_orm::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

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
                (cake_model(1), vec![fruit_model(1, Some(1)),]),
                (
                    cake_model(2),
                    vec![
                        fruit_model(2, Some(2)),
                        fruit_model(3, Some(2)),
                        fruit_model(4, Some(2)),
                    ]
                ),
            ]
        );

        Ok(())
    }

    #[smol_potat::test]
    pub async fn with_related_empty() -> Result<(), sea_orm::DbErr> {
        use sea_orm::tests_cfg::*;
        use sea_orm::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

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
                (cake_model(1), vec![fruit_model(1, Some(1)),]),
                (
                    cake_model(2),
                    vec![
                        fruit_model(2, Some(2)),
                        fruit_model(3, Some(2)),
                        fruit_model(4, Some(2)),
                    ]
                ),
                (cake_model(3), vec![])
            ]
        );

        Ok(())
    }

    #[smol_potat::test]
    pub async fn with_related_many_to_many() -> Result<(), sea_orm::DbErr> {
        use sea_orm::tests_cfg::*;
        use sea_orm::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

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
    pub async fn with_related_many_to_many_2() -> Result<(), sea_orm::DbErr> {
        use sea_orm::tests_cfg::*;
        use sea_orm::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

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
    pub async fn also_linked_base() -> Result<(), sea_orm::DbErr> {
        use sea_orm::tests_cfg::*;
        use sea_orm::{DbBackend, EntityTrait, MockDatabase, Statement, Transaction};

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
    pub async fn also_linked_same_cake() -> Result<(), sea_orm::DbErr> {
        use sea_orm::tests_cfg::*;
        use sea_orm::{DbBackend, EntityTrait, MockDatabase};

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
    pub async fn also_linked_same_vendor() -> Result<(), sea_orm::DbErr> {
        use sea_orm::tests_cfg::*;
        use sea_orm::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

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
    pub async fn also_linked_many_to_many() -> Result<(), sea_orm::DbErr> {
        use sea_orm::tests_cfg::*;
        use sea_orm::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

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
    pub async fn also_linked_empty() -> Result<(), sea_orm::DbErr> {
        use sea_orm::tests_cfg::*;
        use sea_orm::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

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
    pub async fn with_linked_base() -> Result<(), sea_orm::DbErr> {
        use sea_orm::tests_cfg::*;
        use sea_orm::{DbBackend, EntityTrait, MockDatabase, Statement, Transaction};

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
    pub async fn with_linked_same_vendor() -> Result<(), sea_orm::DbErr> {
        use sea_orm::tests_cfg::*;
        use sea_orm::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

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
    pub async fn with_linked_empty() -> Result<(), sea_orm::DbErr> {
        use sea_orm::tests_cfg::*;
        use sea_orm::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

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
    pub async fn with_linked_repeated() -> Result<(), sea_orm::DbErr> {
        use sea_orm::tests_cfg::*;
        use sea_orm::{DbBackend, EntityTrait, IntoMockRow, MockDatabase};

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
}
