use crate::{
    ColumnTrait, ColumnType, EntityTrait, EnumIter, Iden, Identity, IntoIdentity, ModelTrait,
    QueryResult, Related, RelationDef, RelationTrait, Select, TypeErr,
};

#[derive(Default, Debug, Iden)]
#[iden = "cake"]
pub struct Entity;

#[derive(Debug, Default, PartialEq)]
pub struct Model {
    pub id: i32,
    pub name: String,
}

#[derive(Copy, Clone, Debug, Iden, EnumIter)]
pub enum Column {
    Id,
    Name,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Fruit,
}

impl EntityTrait for Entity {
    type Model = Model;

    type Column = Column;

    type Relation = Relation;

    fn primary_key() -> Identity {
        Column::Id.into_identity()
    }
}

impl ModelTrait for Model {
    fn from_query_result(row: QueryResult) -> Result<Self, TypeErr> {
        Ok(Self {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
        })
    }
}

impl ColumnTrait for Column {
    type Entity = Entity;

    fn def(&self) -> ColumnType {
        match self {
            Self::Id => ColumnType::Integer(None),
            Self::Name => ColumnType::String(None),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Fruit => Entity::has_many(super::fruit::Entity)
                .from(Column::Id)
                .to(super::fruit::Column::CakeId)
                .into(),
        }
    }
}

impl Related<super::fruit::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Fruit.def()
    }
}

impl Entity {
    pub fn find_fruit() -> Select<super::fruit::Entity> {
        Self::find_related()
    }
}
