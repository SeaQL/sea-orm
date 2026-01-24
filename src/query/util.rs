use crate::{
    ActiveModelTrait, ColumnTrait, Condition, DbBackend, DbErr, EntityTrait, ExprTrait, IdenStatic,
    Identity, ModelTrait, Value,
};
use sea_query::{ColumnRef, DynIden, Expr, IntoColumnRef, TableRef, ValueTuple};
use std::str::FromStr;

#[derive(Default)]
pub struct ValueTupleBuilder(Option<ValueTuple>);

impl ValueTupleBuilder {
    pub fn push(&mut self, value: Value) {
        match self.0.take() {
            None => {
                self.0 = Some(ValueTuple::One(value));
            }
            Some(ValueTuple::One(a)) => {
                self.0 = Some(ValueTuple::Two(a, value));
            }
            Some(ValueTuple::Two(a, b)) => {
                self.0 = Some(ValueTuple::Three(a, b, value));
            }
            Some(ValueTuple::Three(a, b, c)) => {
                self.0 = Some(ValueTuple::Many(vec![a, b, c, value]));
            }
            Some(ValueTuple::Many(mut items)) => {
                items.push(value);
                self.0 = Some(ValueTuple::Many(items));
            }
        }
    }

    pub fn into_inner(self) -> Option<ValueTuple> {
        self.0
    }
}

pub fn get_key_from_model<Model>(columns: &Identity, model: &Model) -> Result<ValueTuple, DbErr>
where
    Model: ModelTrait,
{
    let mut values = ValueTupleBuilder::default();

    for col in columns.iter() {
        let col_name = col.inner();
        let column = <<Model::Entity as EntityTrait>::Column as FromStr>::from_str(&col_name)
            .map_err(|_| DbErr::Type(format!("Failed at mapping '{col_name}' to column")))?;
        values.push(model.get(column));
    }

    match values.into_inner() {
        Some(values) => Ok(values),
        None => Err(DbErr::Type("Identity zero?".into())),
    }
}

pub fn get_key_from_active_model<ActiveModel>(
    columns: &Identity,
    model: &ActiveModel,
) -> Result<ValueTuple, DbErr>
where
    ActiveModel: ActiveModelTrait,
{
    let mut values = ValueTupleBuilder::default();

    for col in columns.iter() {
        let col_name = col.inner();
        let column = <<ActiveModel::Entity as EntityTrait>::Column as FromStr>::from_str(&col_name)
            .map_err(|_| DbErr::Type(format!("Failed at mapping '{col_name}' to column")))?;
        values.push(match model.get(column).into_value() {
            Some(value) => value,
            None => {
                return Err(DbErr::AttrNotSet(format!(
                    "{}.{}",
                    <ActiveModel::Entity as Default>::default().as_str(),
                    col_name
                )));
            }
        });
    }

    match values.into_inner() {
        Some(values) => Ok(values),
        None => Err(DbErr::Type("Identity zero?".into())),
    }
}

pub fn set_key_on_active_model<ActiveModel>(
    columns: &Identity,
    model: &mut ActiveModel,
    values: ValueTuple,
) -> Result<(), DbErr>
where
    ActiveModel: ActiveModelTrait,
{
    if values.arity() != columns.arity() {
        return Err(DbErr::Type(format!(
            "Arity mismatch: {} != {}",
            values.arity(),
            columns.arity(),
        )));
    }

    for (column, value) in columns.iter().zip(values) {
        let col_name = column.inner();
        let column = <<ActiveModel::Entity as EntityTrait>::Column as FromStr>::from_str(&col_name)
            .map_err(|_| DbErr::Type(format!("Failed at mapping '{col_name}' to column")))?;
        model.set_if_not_equals(column, value);
    }

    Ok(())
}

/// Set null on the key columns. Return true if succeeded, false if column is not nullable.
pub fn clear_key_on_active_model<ActiveModel>(
    columns: &Identity,
    model: &mut ActiveModel,
) -> Result<bool, DbErr>
where
    ActiveModel: ActiveModelTrait,
{
    for col in columns.iter() {
        let col_name = col.inner();
        let column = <<ActiveModel::Entity as EntityTrait>::Column as FromStr>::from_str(&col_name)
            .map_err(|_| DbErr::Type(format!("Failed at mapping '{col_name}' to column")))?;
        if !column.def().is_null() {
            return Ok(false);
        }
        model.set(
            column,
            match model.get(column).into_value() {
                Some(value) => value.as_null(),
                None => {
                    return Err(DbErr::AttrNotSet(format!(
                        "{}.{}",
                        <ActiveModel::Entity as Default>::default().as_str(),
                        col_name
                    )));
                }
            },
        );
    }

    Ok(true)
}

/// Constructs a `WHERE (c1, c2, ...) IN ((v11, v12, ...), (v21, v22, ...), ...)` expression.
/// Degenerates to `WHERE col IN (v1, v2, ...)` when arity = 1.
pub fn column_tuple_in_condition(
    table: &TableRef,
    to: &Identity,
    keys: &[ValueTuple],
    backend: DbBackend,
) -> Result<Condition, DbErr> {
    use itertools::Itertools;

    let arity = to.arity();
    let keys = keys.iter().unique();

    if arity == 1 {
        let values = keys
            .map(|key| match key {
                ValueTuple::One(v) => Ok(Expr::val(v.to_owned())),
                _ => Err(arity_mismatch(arity, key)),
            })
            .collect::<Result<Vec<_>, DbErr>>()?;

        let expr = Expr::col(table_column(
            table,
            to.iter().next().expect("Checked above"),
        ))
        .is_in(values);

        Ok(expr.into())
    } else if cfg!(feature = "sqlite-no-row-value-before-3_15")
        && matches!(backend, DbBackend::Sqlite)
    {
        // SQLite supports row value expressions since 3.15.0
        // https://www.sqlite.org/releaselog/3_15_0.html

        let table_columns = create_table_columns(table, to);

        let mut outer = Condition::any();

        for key in keys {
            let key_arity = key.arity();
            if arity != key_arity {
                return Err(arity_mismatch(arity, key));
            }

            let table_columns = table_columns.iter().cloned();
            let values = key.clone().into_iter().map(Expr::val);

            let inner = table_columns
                .zip(values)
                .fold(Condition::all(), |cond, (column, value)| {
                    cond.add(column.eq(value))
                });

            // Build `(c1 = v11 AND c2 = v12) OR (c1 = v21 AND c2 = v22) ...`
            outer = outer.add(inner);
        }

        Ok(outer)
    } else {
        let table_columns = create_table_columns(table, to);

        // A vector of tuples of values, e.g. [(v11, v12, ...), (v21, v22, ...), ...]
        let value_tuples = keys
            .map(|key| {
                let key_arity = key.arity();
                if arity != key_arity {
                    return Err(arity_mismatch(arity, key));
                }

                let tuple_exprs = key.clone().into_iter().map(Expr::val);

                Ok(Expr::tuple(tuple_exprs))
            })
            .collect::<Result<Vec<_>, DbErr>>()?;

        // Build `(c1, c2, ...) IN ((v11, v12, ...), (v21, v22, ...), ...)`
        let expr = Expr::tuple(table_columns).is_in(value_tuples);

        Ok(expr.into())
    }
}

fn arity_mismatch(expected: usize, actual: &ValueTuple) -> DbErr {
    DbErr::Type(format!(
        "Loader: arity mismatch: expected {expected}, got {} in {actual:?}",
        actual.arity()
    ))
}

fn table_column(tbl: &TableRef, col: &DynIden) -> ColumnRef {
    (tbl.sea_orm_table().to_owned(), col.clone()).into_column_ref()
}

/// Create a vector of `Expr::col` from the table and identity, e.g. [Expr::col((table, col1)), Expr::col((table, col2)), ...]
fn create_table_columns(table: &TableRef, cols: &Identity) -> Vec<Expr> {
    cols.iter()
        .map(|col| table_column(table, col))
        .map(Expr::col)
        .collect()
}
