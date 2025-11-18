use crate::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, IdenStatic, Identity, ModelTrait, Value,
};
use sea_query::ValueTuple;
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
