use crate::{
    Condition, ConnectionTrait, DbErr, EntityTrait, Identity, JoinType, ModelTrait, QueryFilter,
    QuerySelect, Related, RelationType, Select, dynamic, error::*,
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
        <Self::Model as ModelTrait>::Entity: Related<R>;

    /// Used to eager load has_many relations
    async fn load_many<R, S, C>(&self, stmt: S, db: &C) -> Result<Vec<Vec<R::Model>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        <Self::Model as ModelTrait>::Entity: Related<R>;

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
        <Self::Model as ModelTrait>::Entity: Related<R>;
}

/// This trait implements the Data Loader API for nested loading
#[async_trait]
pub trait NestedLoaderTrait {
    /// Source model
    type Model: ModelTrait;

    /// Used to eager load has_one relations
    async fn load_one<R, S, C>(&self, stmt: S, db: &C) -> Result<Vec<Vec<Option<R::Model>>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        <Self::Model as ModelTrait>::Entity: Related<R>;

    /// Used to eager load has_many relations
    async fn load_many<R, S, C>(&self, stmt: S, db: &C) -> Result<Vec<Vec<Vec<R::Model>>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        <Self::Model as ModelTrait>::Entity: Related<R>;
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
        <Self::Model as ModelTrait>::Entity: Related<R>,
    {
        self.as_slice().load_one(stmt, db).await
    }

    async fn load_many<R, S, C>(&self, stmt: S, db: &C) -> Result<Vec<Vec<R::Model>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        <Self::Model as ModelTrait>::Entity: Related<R>,
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
        <Self::Model as ModelTrait>::Entity: Related<R>,
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
        <Self::Model as ModelTrait>::Entity: Related<R>,
    {
        let rel_def = <<Self::Model as ModelTrait>::Entity as Related<R>>::to();
        if rel_def.rel_type != RelationType::HasOne {
            return Err(query_err("Relation is HasMany instead of HasOne"));
        }
        loader_impl(self.iter(), stmt.select(), db).await
    }

    async fn load_many<R, S, C>(&self, stmt: S, db: &C) -> Result<Vec<Vec<R::Model>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        <Self::Model as ModelTrait>::Entity: Related<R>,
    {
        loader_impl(self.iter(), stmt.select(), db).await
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
        <Self::Model as ModelTrait>::Entity: Related<R>,
    {
        if let Some(via_rel) = <<Self::Model as ModelTrait>::Entity as Related<R>>::via() {
            let rel_def = <<Self::Model as ModelTrait>::Entity as Related<R>>::to();
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

            let stmt = QueryFilter::filter(stmt.select(), condition);

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

#[async_trait]
impl<M> LoaderTrait for &[Option<M>]
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
        <Self::Model as ModelTrait>::Entity: Related<R>,
    {
        let rel_def = <<Self::Model as ModelTrait>::Entity as Related<R>>::to();
        if rel_def.rel_type != RelationType::HasOne {
            return Err(query_err("Relation is HasMany instead of HasOne"));
        }
        let items: Vec<Option<R::Model>> =
            loader_impl(self.iter().filter_map(|o| o.as_ref()), stmt.select(), db).await?;
        Ok(assemble_options(self, items))
    }

    async fn load_many<R, S, C>(&self, stmt: S, db: &C) -> Result<Vec<Vec<R::Model>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        <Self::Model as ModelTrait>::Entity: Related<R>,
    {
        let items: Vec<Vec<R::Model>> =
            loader_impl(self.iter().filter_map(|o| o.as_ref()), stmt.select(), db).await?;
        Ok(assemble_options(self, items))
    }

    /// Not implemented. Please use load_many
    async fn load_many_to_many<R, S, V, C>(
        &self,
        _stmt: S,
        _via: V,
        _db: &C,
    ) -> Result<Vec<Vec<R::Model>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        V: EntityTrait,
        V::Model: Send + Sync,
        <Self::Model as ModelTrait>::Entity: Related<R>,
    {
        unimplemented!("Please use load_many")
    }
}

fn assemble_options<I, T: Default>(input: &[Option<I>], items: Vec<T>) -> Vec<T> {
    let mut items = items.into_iter();
    let mut output = Vec::new();
    for input in input.iter() {
        if input.is_some() {
            output.push(items.next().unwrap_or_default());
        } else {
            output.push(T::default());
        }
    }
    output
}

#[async_trait]
impl<M> NestedLoaderTrait for &[Vec<M>]
where
    M: ModelTrait + Sync,
{
    type Model = M;

    async fn load_one<R, S, C>(&self, stmt: S, db: &C) -> Result<Vec<Vec<Option<R::Model>>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        <Self::Model as ModelTrait>::Entity: Related<R>,
    {
        let rel_def = <<Self::Model as ModelTrait>::Entity as Related<R>>::to();
        if rel_def.rel_type != RelationType::HasOne {
            return Err(query_err("Relation is HasMany instead of HasOne"));
        }
        let items: Vec<Option<R::Model>> =
            loader_impl(self.iter().flatten(), stmt.select(), db).await?;
        Ok(assemble_vectors(self, items))
    }

    async fn load_many<R, S, C>(&self, stmt: S, db: &C) -> Result<Vec<Vec<Vec<R::Model>>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        <Self::Model as ModelTrait>::Entity: Related<R>,
    {
        let items: Vec<Vec<R::Model>> =
            loader_impl(self.iter().flatten(), stmt.select(), db).await?;
        Ok(assemble_vectors(self, items))
    }
}

fn assemble_vectors<I, T: Default>(input: &[Vec<I>], items: Vec<T>) -> Vec<Vec<T>> {
    let mut items = items.into_iter();

    let mut output = Vec::new();

    for input in input.iter() {
        output.push(Vec::new());

        for _inner in input.iter() {
            output
                .last_mut()
                .unwrap()
                .push(items.next().unwrap_or_default());
        }
    }

    output
}

trait Container: Default + Clone {
    type Item;
    fn add(&mut self, item: Self::Item);
}

impl<T: Clone> Container for Vec<T> {
    type Item = T;
    fn add(&mut self, item: Self::Item) {
        self.push(item);
    }
}

impl<T: Clone> Container for Option<T> {
    type Item = T;
    fn add(&mut self, item: Self::Item) {
        self.replace(item);
    }
}

async fn loader_impl<'a, Model, Iter, R, C, T>(
    items: Iter,
    stmt: Select<R>,
    db: &C,
) -> Result<Vec<T>, DbErr>
where
    Model: ModelTrait + Sync + 'a,
    Iter: Iterator<Item = &'a Model> + 'a,
    C: ConnectionTrait,
    R: EntityTrait,
    R::Model: Send + Sync,
    Model::Entity: Related<R>,
    T: Container<Item = R::Model>,
{
    let (keys, hashmap) = if let Some(via_def) = <Model::Entity as Related<R>>::via() {
        let keys = items
            .map(|model| extract_key(&via_def.from_col, model))
            .collect::<Result<Vec<_>, _>>()?;

        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let condition = prepare_condition(&via_def.to_tbl, &via_def.to_col, &keys);

        let stmt = QueryFilter::filter(
            stmt.join_rev(JoinType::InnerJoin, <Model::Entity as Related<R>>::to()),
            condition,
        );

        // The idea is to do a SelectTwo with join, then extract key via a dynamic model
        // i.e. select (baker + cake_baker) and extract cake_id from result rows
        // SELECT "baker"."id", "baker"."name", "baker"."contact_details", "baker"."bakery_id",
        //     "cakes_bakers"."cake_id" <- extra select
        // FROM "baker" <- target
        // INNER JOIN "cakes_bakers" <- junction
        //     ON "cakes_bakers"."baker_id" = "baker"."id" <- relation
        // WHERE "cakes_bakers"."cake_id" IN (..)

        let data = stmt
            .select_also_dyn_model(
                via_def.to_tbl.sea_orm_table().clone(),
                dynamic::ModelType {
                    // we uses the left Model's type but the right Model's field
                    fields: extract_col_type::<Model>(&via_def.from_col, &via_def.to_col)?,
                },
            )
            .all(db)
            .await?;

        let mut hashmap: HashMap<ValueTuple, T> =
            keys.iter()
                .fold(HashMap::new(), |mut acc, key: &ValueTuple| {
                    acc.insert(key.clone(), T::default());
                    acc
                });

        for (item, key) in data {
            let key = dyn_model_to_key(key)?;

            let vec = hashmap.get_mut(&key).ok_or_else(|| {
                DbErr::RecordNotFound(format!("Loader: failed to find model for {key:?}"))
            })?;

            vec.add(item);
        }

        (keys, hashmap)
    } else {
        let rel_def = <Model::Entity as Related<R>>::to();

        let keys = items
            .map(|model| extract_key(&rel_def.from_col, model))
            .collect::<Result<Vec<_>, _>>()?;

        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let condition = prepare_condition(&rel_def.to_tbl, &rel_def.to_col, &keys);

        let stmt = QueryFilter::filter(stmt, condition);

        let data = stmt.all(db).await?;

        let mut hashmap: HashMap<ValueTuple, T> = Default::default();

        for item in data {
            let key = extract_key(&rel_def.to_col, &item)?;
            if !hashmap.contains_key(&key) {
                let mut holder = T::default();
                holder.add(item);
                hashmap.insert(key, holder);
            } else {
                let holder = hashmap.get_mut(&key).ok_or_else(|| {
                    DbErr::RecordNotFound(format!("Loader: failed to find model for {key:?}"))
                })?;
                holder.add(item);
            }
        }

        (keys, hashmap)
    };

    let result: Vec<T> = keys
        .iter()
        .map(|key: &ValueTuple| hashmap.get(key).cloned().unwrap_or_default())
        .collect();

    Ok(result)
}

fn cmp_table_ref(left: &TableRef, right: &TableRef) -> bool {
    left == right
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

fn extract_col_type<Model>(
    left: &Identity,
    right: &Identity,
) -> Result<Vec<dynamic::FieldType>, DbErr>
where
    Model: ModelTrait,
{
    Ok(match (left, right) {
        (Identity::Unary(a), Identity::Unary(aa)) => {
            let col_a =
                <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(
                    &a.inner(),
                )
                .map_err(|_| DbErr::Type(format!("Failed at mapping '{a}'")))?;
            vec![dynamic::FieldType::new(
                aa.clone(),
                Model::get_value_type(col_a),
            )]
        }
        (Identity::Binary(a, b), Identity::Binary(aa, bb)) => {
            let col_a =
                <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(
                    &a.inner(),
                )
                .map_err(|_| DbErr::Type(format!("Failed at mapping '{a}'")))?;
            let col_b =
                <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(
                    &b.inner(),
                )
                .map_err(|_| DbErr::Type(format!("Failed at mapping '{b}'")))?;
            vec![
                dynamic::FieldType::new(aa.clone(), Model::get_value_type(col_a)),
                dynamic::FieldType::new(bb.clone(), Model::get_value_type(col_b)),
            ]
        }
        (Identity::Ternary(a, b, c), Identity::Ternary(aa, bb, cc)) => {
            let col_a =
                <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(
                    &a.inner(),
                )
                .map_err(|_| DbErr::Type(format!("Failed at mapping '{a}'")))?;
            let col_b =
                <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(
                    &b.inner(),
                )
                .map_err(|_| DbErr::Type(format!("Failed at mapping '{b}'")))?;
            let col_c =
                <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(
                    &c.inner(),
                )
                .map_err(|_| DbErr::Type(format!("Failed at mapping '{c}'")))?;
            vec![
                dynamic::FieldType::new(aa.clone(), Model::get_value_type(col_a)),
                dynamic::FieldType::new(bb.clone(), Model::get_value_type(col_b)),
                dynamic::FieldType::new(cc.clone(), Model::get_value_type(col_c)),
            ]
        }
        (Identity::Many(left), Identity::Many(right)) => {
            let mut vec = Vec::new();
            for (a, aa) in left.iter().zip(right) {
                let col_a =
                    <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(
                        &a.inner(),
                    )
                    .map_err(|_| DbErr::Type(format!("Failed at mapping '{a}'")))?;
                vec.push(dynamic::FieldType::new(
                    aa.clone(),
                    Model::get_value_type(col_a),
                ));
            }
            vec
        }
        _ => {
            return Err(DbErr::Type(format!(
                "Identity mismatch: {left:?} != {right:?}"
            )));
        }
    })
}

#[allow(clippy::unwrap_used)]
fn dyn_model_to_key(dyn_model: dynamic::Model) -> Result<ValueTuple, DbErr> {
    Ok(match dyn_model.fields.len() {
        0 => return Err(DbErr::Type("Identity zero?".into())),
        1 => ValueTuple::One(dyn_model.fields.into_iter().next().unwrap().value),
        2 => {
            let mut iter = dyn_model.fields.into_iter();
            ValueTuple::Two(iter.next().unwrap().value, iter.next().unwrap().value)
        }
        3 => {
            let mut iter = dyn_model.fields.into_iter();
            ValueTuple::Three(
                iter.next().unwrap().value,
                iter.next().unwrap().value,
                iter.next().unwrap().value,
            )
        }
        _ => ValueTuple::Many(dyn_model.fields.into_iter().map(|v| v.value).collect()),
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

    #[test]
    fn test_assemble_vectors() {
        use super::assemble_vectors;

        assert_eq!(
            assemble_vectors(&[vec![1], vec![], vec![2, 3], vec![]], vec![11, 22, 33]),
            [vec![11], vec![], vec![22, 33], vec![]]
        );
    }
}
