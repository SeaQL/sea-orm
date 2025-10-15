use crate::{
    ColumnTrait, Condition, ConnectionTrait, DbBackend, DbErr, EntityTrait, Identity, JoinType,
    ModelTrait, QueryFilter, QuerySelect, Related, RelationType, Select, dynamic, error::*,
};
use async_trait::async_trait;
use itertools::Itertools;
use sea_query::{ColumnRef, DynIden, Expr, ExprTrait, IntoColumnRef, TableRef, ValueTuple};
use std::{collections::HashMap, str::FromStr};

// TODO: Replace DynIden::inner with a better API that without clone

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

#[doc(hidden)]
#[async_trait]
pub trait LoaderTraitEx {
    /// Source model
    type Model: ModelTrait;

    async fn load_one_ex<R, S, C>(&self, stmt: S, db: &C) -> Result<Vec<Option<R::ModelEx>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        R::ModelEx: From<R::Model>,
        <Self::Model as ModelTrait>::Entity: Related<R>;

    async fn load_many_ex<R, S, C>(&self, stmt: S, db: &C) -> Result<Vec<Vec<R::ModelEx>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        R::ModelEx: From<R::Model>,
        <Self::Model as ModelTrait>::Entity: Related<R>;
}

#[doc(hidden)]
#[async_trait]
pub trait NestedLoaderTrait {
    /// Source model
    type Model: ModelTrait;

    async fn load_one_ex<R, S, C>(
        &self,
        stmt: S,
        db: &C,
    ) -> Result<Vec<Vec<Option<R::ModelEx>>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        R::ModelEx: From<R::Model>,
        <Self::Model as ModelTrait>::Entity: Related<R>;

    async fn load_many_ex<R, S, C>(
        &self,
        stmt: S,
        db: &C,
    ) -> Result<Vec<Vec<Vec<R::ModelEx>>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        R::ModelEx: From<R::Model>,
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
        LoaderTrait::load_one(&self.as_slice(), stmt, db).await
    }

    async fn load_many<R, S, C>(&self, stmt: S, db: &C) -> Result<Vec<Vec<R::Model>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        <Self::Model as ModelTrait>::Entity: Related<R>,
    {
        LoaderTrait::load_many(&self.as_slice(), stmt, db).await
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
        LoaderTrait::load_many_to_many(&self.as_slice(), stmt, via, db).await
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
                let condition = prepare_condition::<M>(
                    &via_rel.to_tbl,
                    &via_rel.from_col,
                    &via_rel.to_col,
                    &pkeys,
                    db.get_database_backend(),
                )?;
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

            let condition = prepare_condition::<V::Model>(
                &rel_def.to_tbl,
                &rel_def.from_col,
                &rel_def.to_col,
                &keys,
                db.get_database_backend(),
            )?;

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
impl<M> LoaderTraitEx for &[M]
where
    M: ModelTrait + Sync,
{
    type Model = M;

    async fn load_one_ex<R, S, C>(&self, stmt: S, db: &C) -> Result<Vec<Option<R::ModelEx>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        R::ModelEx: From<R::Model>,
        <Self::Model as ModelTrait>::Entity: Related<R>,
    {
        let rel_def = <<Self::Model as ModelTrait>::Entity as Related<R>>::to();
        if rel_def.rel_type != RelationType::HasOne {
            return Err(query_err("Relation is HasMany instead of HasOne"));
        }
        loader_impl(self.iter(), stmt.select(), db).await
    }

    async fn load_many_ex<R, S, C>(&self, stmt: S, db: &C) -> Result<Vec<Vec<R::ModelEx>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        R::ModelEx: From<R::Model>,
        <Self::Model as ModelTrait>::Entity: Related<R>,
    {
        loader_impl(self.iter(), stmt.select(), db).await
    }
}

#[async_trait]
impl<M> LoaderTraitEx for &[Option<M>]
where
    M: ModelTrait + Sync,
{
    type Model = M;

    async fn load_one_ex<R, S, C>(&self, stmt: S, db: &C) -> Result<Vec<Option<R::ModelEx>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        R::ModelEx: From<R::Model>,
        <Self::Model as ModelTrait>::Entity: Related<R>,
    {
        let rel_def = <<Self::Model as ModelTrait>::Entity as Related<R>>::to();
        if rel_def.rel_type != RelationType::HasOne {
            return Err(query_err("Relation is HasMany instead of HasOne"));
        }
        let items: Vec<Option<R::ModelEx>> =
            loader_impl(self.iter().filter_map(|o| o.as_ref()), stmt.select(), db).await?;
        Ok(assemble_options(self, items))
    }

    async fn load_many_ex<R, S, C>(&self, stmt: S, db: &C) -> Result<Vec<Vec<R::ModelEx>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        R::ModelEx: From<R::Model>,
        <Self::Model as ModelTrait>::Entity: Related<R>,
    {
        let items: Vec<Vec<R::ModelEx>> =
            loader_impl(self.iter().filter_map(|o| o.as_ref()), stmt.select(), db).await?;
        Ok(assemble_options(self, items))
    }
}

#[async_trait]
impl<M> NestedLoaderTrait for &[Vec<M>]
where
    M: ModelTrait + Sync,
{
    type Model = M;

    async fn load_one_ex<R, S, C>(
        &self,
        stmt: S,
        db: &C,
    ) -> Result<Vec<Vec<Option<R::ModelEx>>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        R::ModelEx: From<R::Model>,
        <Self::Model as ModelTrait>::Entity: Related<R>,
    {
        let rel_def = <<Self::Model as ModelTrait>::Entity as Related<R>>::to();
        if rel_def.rel_type != RelationType::HasOne {
            return Err(query_err("Relation is HasMany instead of HasOne"));
        }
        let items: Vec<Option<R::ModelEx>> =
            loader_impl(self.iter().flatten(), stmt.select(), db).await?;
        Ok(assemble_vectors(self, items))
    }

    async fn load_many_ex<R, S, C>(
        &self,
        stmt: S,
        db: &C,
    ) -> Result<Vec<Vec<Vec<R::ModelEx>>>, DbErr>
    where
        C: ConnectionTrait,
        R: EntityTrait,
        R::Model: Send + Sync,
        S: EntityOrSelect<R>,
        R::ModelEx: From<R::Model>,
        <Self::Model as ModelTrait>::Entity: Related<R>,
    {
        let items: Vec<Vec<R::ModelEx>> =
            loader_impl(self.iter().flatten(), stmt.select(), db).await?;
        Ok(assemble_vectors(self, items))
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

fn assemble_vectors<I, T: Default>(input: &[Vec<I>], items: Vec<T>) -> Vec<Vec<T>> {
    let mut items = items.into_iter();

    let mut output = Vec::new();

    for input in input.iter() {
        output.push(Vec::new());

        for _inner in input.iter() {
            output
                .last_mut()
                .expect("Pushed above")
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

async fn loader_impl<'a, Model, Iter, R, C, T, Output>(
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
    Output: From<R::Model>,
    T: Container<Item = Output>,
{
    let (keys, hashmap) = if let Some(via_def) = <Model::Entity as Related<R>>::via() {
        let keys = items
            .map(|model| extract_key(&via_def.from_col, model))
            .collect::<Result<Vec<_>, _>>()?;

        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let condition = prepare_condition::<Model>(
            &via_def.to_tbl,
            &via_def.from_col,
            &via_def.to_col,
            &keys,
            db.get_database_backend(),
        )?;

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

            vec.add(item.into());
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

        let condition = prepare_condition::<Model>(
            &rel_def.to_tbl,
            &rel_def.from_col,
            &rel_def.to_col,
            &keys,
            db.get_database_backend(),
        )?;

        let stmt = QueryFilter::filter(stmt, condition);

        let data = stmt.all(db).await?;

        let mut hashmap: HashMap<ValueTuple, T> = Default::default();

        for item in data {
            let key = extract_key(&rel_def.to_col, &item)?;
            let holder = hashmap.entry(key).or_default();
            holder.add(item.into());
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
    let values = target_col
        .iter()
        .map(|col| {
            let col_name = col.inner();
            let column =
                <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(
                    &col_name,
                )
                .map_err(|_| DbErr::Type(format!("Failed at mapping '{col_name}' to column")))?;
            Ok(model.get(column))
        })
        .collect::<Result<Vec<_>, DbErr>>()?;

    Ok(match values.len() {
        0 => return Err(DbErr::Type("Identity zero?".into())),
        1 => ValueTuple::One(values.into_iter().next().expect("checked")),
        2 => {
            let mut it = values.into_iter();
            ValueTuple::Two(it.next().expect("checked"), it.next().expect("checked"))
        }
        3 => {
            let mut it = values.into_iter();
            ValueTuple::Three(
                it.next().expect("checked"),
                it.next().expect("checked"),
                it.next().expect("checked"),
            )
        }
        _ => ValueTuple::Many(values),
    })
}

fn extract_col_type<Model>(
    left: &Identity,
    right: &Identity,
) -> Result<Vec<dynamic::FieldType>, DbErr>
where
    Model: ModelTrait,
{
    if left.arity() != right.arity() {
        return Err(DbErr::Type(format!(
            "Identity mismatch: left: {} != right: {}",
            left.arity(),
            right.arity()
        )));
    }

    let vec = left
        .iter()
        .zip_eq(right.iter())
        .map(|(a, aa)| {
            let col_a =
                <<<Model as ModelTrait>::Entity as EntityTrait>::Column as FromStr>::from_str(
                    &a.inner(),
                )
                .map_err(|_| DbErr::Type(format!("Failed at mapping '{a}'")))?;
            Ok(dynamic::FieldType::new(
                aa.clone(),
                Model::get_value_type(col_a),
            ))
        })
        .collect::<Result<Vec<_>, DbErr>>()?;

    Ok(vec)
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

// Perf: Perhaps we could lift the is_postgres check and then implement a simpler method for other database backends that doesn't require type casting.
// But in any case, these computations are very lightweight.
fn prepare_condition<Model>(
    table: &TableRef,
    from: &Identity,
    to: &Identity,
    keys: &[ValueTuple],
    backend: DbBackend,
) -> Result<Condition, DbErr>
where
    Model: ModelTrait,
{
    fn arity_mismatch(expected: usize, actual: &ValueTuple) -> DbErr {
        DbErr::Type(format!(
            "Loader: arity mismatch: expected {expected}, got {} in {actual:?}",
            actual.arity()
        ))
    }

    let keys = keys.iter().unique();
    let column_pairs = resolve_column_pairs::<Model>(table, from, to)?;

    if column_pairs.is_empty() {
        return Err(DbErr::Type(format!(
            "Loader: resolved zero columns for identities {from:?} -> {to:?}"
        )));
    }

    let arity = column_pairs.len();

    if cfg!(not(feature = "sqlite-3_15")) && matches!(backend, DbBackend::Sqlite) {
        // SQLite supports row value expressions since 3.15.0
        // https://www.sqlite.org/releaselog/3_15_0.html
        let mut outer = Condition::any();
        for key in keys {
            let key_arity = key.arity();
            if arity != key_arity {
                return Err(arity_mismatch(arity, key));
            }

            let mut inner = Condition::all();
            for ((column_ref, _), value) in column_pairs.iter().zip(key.clone().into_iter()) {
                inner = inner.add(Expr::col(column_ref.clone()).eq(Expr::val(value)));
            }

            outer = outer.add(inner);
        }

        Ok(outer)
    } else {
        // Build `(c1, c2, ...) IN ((v11, v12, ...), (v21, v22, ...), ...)`
        let values = keys
            .map(|key| {
                let key_arity = key.arity();
                if arity != key_arity {
                    return Err(arity_mismatch(arity, key));
                }

                // For Postgres, we need to use `AS` to cast the value to the correct type
                let tuple_exprs: Vec<_> = if matches!(backend, DbBackend::Postgres) {
                    key.clone()
                        .into_iter()
                        .zip(column_pairs.iter().map(|(_, model_column)| model_column))
                        .map(|(v, model_column)| model_column.save_as(Expr::val(v)))
                        .collect()
                } else {
                    key.clone().into_iter().map(Expr::val).collect()
                };

                Ok(Expr::tuple(tuple_exprs))
            })
            .collect::<Result<Vec<_>, DbErr>>()?;

        let expr = Expr::tuple(
            column_pairs
                .iter()
                .map(|(column_ref, _)| Expr::col(column_ref.clone())),
        )
        .is_in(values);

        Ok(expr.into())
    }
}

type ColumnPairs<M> = Vec<(
    ColumnRef,
    <<M as ModelTrait>::Entity as EntityTrait>::Column,
)>;

fn resolve_column_pairs<Model>(
    table: &TableRef,
    from: &Identity,
    to: &Identity,
) -> Result<ColumnPairs<Model>, DbErr>
where
    Model: ModelTrait,
    <<Model as ModelTrait>::Entity as EntityTrait>::Column: ColumnTrait + Clone,
{
    let from_columns = parse_identity_columns::<Model>(from)?;
    let to_columns = column_refs_from_identity(table, to);

    if from_columns.len() != to_columns.len() {
        return Err(DbErr::Type(format!(
            "Loader: identity column count mismatch between {from:?} and {to:?}"
        )));
    }

    Ok(to_columns.into_iter().zip(from_columns).collect())
}

fn column_refs_from_identity(table: &TableRef, identity: &Identity) -> Vec<ColumnRef> {
    identity
        .iter()
        .map(|col| table_column(table, col))
        .collect()
}

fn parse_identity_columns<Model>(
    identity: &Identity,
) -> Result<Vec<<<Model as ModelTrait>::Entity as EntityTrait>::Column>, DbErr>
where
    Model: ModelTrait,
{
    identity
        .iter()
        .map(|from_col| try_conv_ident_to_column::<Model>(from_col))
        .collect()
}

fn try_conv_ident_to_column<Model>(
    ident: &DynIden,
) -> Result<<<Model as ModelTrait>::Entity as EntityTrait>::Column, DbErr>
where
    Model: ModelTrait,
{
    let column_name = ident.inner();
    <<Model as ModelTrait>::Entity as EntityTrait>::Column::from_str(&column_name)
        .map_err(|_| DbErr::Type(format!("Failed at mapping '{column_name}' to column")))
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
