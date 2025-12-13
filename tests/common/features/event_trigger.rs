use sea_orm::entity::prelude::*;
use sea_orm::{
    TryGetError, TryGetable,
    sea_query::{ArrayType, ColumnType, ValueType},
};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "event_trigger")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub events: Events,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Events(pub Vec<Option<Event>>);

impl From<Events> for Value {
    fn from(events: Events) -> Self {
        let Events(events) = events;
        let arr: Vec<Option<String>> = events.into_iter().map(|opt| opt.map(|it| it.0)).collect();
        Value::from(arr)
    }
}

impl TryGetable for Events {
    fn try_get_by<I: sea_orm::ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
        let vec: Vec<Option<String>> = res.try_get_by(idx).map_err(TryGetError::DbErr)?;
        let events = vec.into_iter().map(|opt| opt.map(Event)).collect();
        Ok(Events(events))
    }
}

impl ValueType for Events {
    fn try_from(v: Value) -> Result<Self, sea_query::ValueTypeErr> {
        let vec: Vec<Option<String>> = <Vec<Option<String>> as ValueType>::try_from(v)?;
        let events = vec.into_iter().map(|opt| opt.map(Event)).collect();
        Ok(Events(events))
    }

    fn type_name() -> String {
        stringify!(Events).to_owned()
    }

    fn array_type() -> ArrayType {
        ArrayType::String
    }

    fn column_type() -> ColumnType {
        ColumnType::Array(RcOrArc::new(ColumnType::String(StringLen::None)))
    }
}
