use crate::{
    ColumnTrait, ColumnType, EntityTrait, EnumIter, Iden, IdenStatic, PrimaryKeyOfModel,
    ModelTrait, QueryResult, Related, RelationDef, RelationTrait, Select, TypeErr, Value, PrimaryKeyTrait
};

#[derive(Copy, Clone, Default, Debug)]
pub struct Entity;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Model {
    pub id: i32,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Column {
    Id,
    Name,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum PrimaryKey {
    Id,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Fruit,
}

impl EntityTrait for Entity {
    type Model = Model;

    type Column = Column;

    type PrimaryKey = PrimaryKey;

    type Relation = Relation;
}

// TODO: implement with derive macro
impl Iden for Entity {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "{}", self.as_str()).unwrap();
    }
}

// TODO: implement with derive macro
impl IdenStatic for Entity {
    fn as_str(&self) -> &str {
        "cake"
    }
}

// TODO: implement with derive macro
impl ModelTrait for Model {
    type Column = Column;

    fn get(&self, c: Self::Column) -> Value {
        match c {
            Column::Id => self.id.clone().into(),
            Column::Name => self.name.clone().into(),
        }
    }

    fn set(&mut self, c: Self::Column, v: Value) {
        match c {
            Column::Id => self.id = v.unwrap(),
            Column::Name => self.name = v.unwrap(),
        }
    }

    fn from_query_result(row: QueryResult) -> Result<Self, TypeErr> {
        Ok(Self {
            id: row.try_get(Column::Id.as_str())?,
            name: row.try_get(Column::Name.as_str())?,
        })
    }
}

// TODO: implement with derive macro
impl Iden for Column {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "{}", self.as_str()).unwrap();
    }
}

// TODO: implement with derive macro
impl IdenStatic for Column {
    fn as_str(&self) -> &str {
        match self {
            Self::Id => "id",
            Self::Name => "name",
        }
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


// TODO: implement with derive macro
impl Iden for PrimaryKey {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "{}", self.as_str()).unwrap();
    }
}

// TODO: implement with derive macro
impl IdenStatic for PrimaryKey {
    fn as_str(&self) -> &str {
        match self {
            Self::Id => "id",
        }
    }
}

// TODO: implement with derive macro
impl PrimaryKeyTrait for PrimaryKey {}

// TODO: implement with derive macro
impl PrimaryKeyOfModel<Model> for PrimaryKey {
    fn into_column(self) -> <Model as ModelTrait>::Column {
        match self {
            Self::Id => Column::Id,
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

impl Model {
    pub fn find_fruit(&self) -> Select<super::fruit::Entity> {
        Entity::find_related().belongs_to::<Entity>(self)
    }
}
