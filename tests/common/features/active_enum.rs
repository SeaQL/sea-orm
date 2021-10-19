use sea_orm::{entity::prelude::*, TryGetError, TryGetable};
use sea_query::{Nullable, ValueType, ValueTypeErr};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "active_enum")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub category: Category,
    pub category_opt: Option<Category>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, PartialEq)]
pub enum Category {
    Big,
    Small,
}

impl ActiveEnum for Category {
    type Value = String;

    fn to_value(&self) -> Self::Value {
        match self {
            Self::Big => "B",
            Self::Small => "S",
        }
        .to_owned()
    }

    fn try_from_value(v: &Self::Value) -> Result<Self, DbErr> {
        match v.as_ref() {
            "B" => Ok(Self::Big),
            "S" => Ok(Self::Small),
            _ => Err(DbErr::Query(format!(
                "unexpected value for {} enum: {}",
                stringify!(Category),
                v
            ))),
        }
    }

    fn db_type() -> ColumnDef {
        ColumnType::String(Some(1)).def()
    }
}

impl Into<Value> for Category {
    fn into(self) -> Value {
        self.to_value().into()
    }
}

impl TryGetable for Category {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let value = <<Self as ActiveEnum>::Value as TryGetable>::try_get(res, pre, col)?;
        Self::try_from_value(&value).map_err(|e| TryGetError::DbErr(e))
    }
}

impl ValueType for Category {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        let value = <<Self as ActiveEnum>::Value as ValueType>::try_from(v)?;
        Self::try_from_value(&value).map_err(|_| ValueTypeErr)
    }

    fn type_name() -> String {
        <<Self as ActiveEnum>::Value as ValueType>::type_name()
    }

    fn column_type() -> sea_query::ColumnType {
        <Self as ActiveEnum>::db_type()
            .get_column_type()
            .to_owned()
            .into()
    }
}

impl Nullable for Category {
    fn null() -> Value {
        <<Self as ActiveEnum>::Value as Nullable>::null()
    }
}
