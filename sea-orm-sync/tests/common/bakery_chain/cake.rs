use sea_orm::{ConnectionTrait, entity::prelude::*};

#[sea_orm::compact_model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "cake")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    #[sea_orm(column_type = "Decimal(Some((16, 4)))")]
    pub price: Decimal,
    pub bakery_id: Option<i32>,
    pub gluten_free: bool,
    pub serial: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::bakery::Entity",
        from = "Column::BakeryId",
        to = "super::bakery::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    Bakery,
    #[sea_orm(has_many = "super::lineitem::Entity")]
    Lineitem,
}

impl Related<super::bakery::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bakery.def()
    }
}

impl Related<super::baker::Entity> for Entity {
    fn to() -> RelationDef {
        super::cakes_bakers::Relation::Baker.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::cakes_bakers::Relation::Cake.def().rev())
    }
}

impl Related<super::lineitem::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Lineitem.def()
    }
}

pub struct ToBakery;
impl Linked for ToBakery {
    type FromEntity = super::cake::Entity;
    type ToEntity = super::bakery::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::Bakery.def()]
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        use sea_orm::Set;
        Self {
            serial: Set(Uuid::new_v4()),
            ..ActiveModelTrait::default()
        }
    }

    fn before_save<C>(self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if self.price.as_ref() == &Decimal::ZERO {
            Err(DbErr::Custom(format!(
                "[before_save] Invalid Price, insert: {insert}"
            )))
        } else {
            Ok(self)
        }
    }

    fn after_save<C>(model: Model, _db: &C, insert: bool) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        if model.price < Decimal::ZERO {
            Err(DbErr::Custom(format!(
                "[after_save] Invalid Price, insert: {insert}"
            )))
        } else {
            Ok(model)
        }
    }

    fn before_delete<C>(self, _db: &C) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if self.name.as_ref().contains("(err_on_before_delete)") {
            Err(DbErr::Custom(
                "[before_delete] Cannot be deleted".to_owned(),
            ))
        } else {
            Ok(self)
        }
    }

    fn after_delete<C>(self, _db: &C) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if self.name.as_ref().contains("(err_on_after_delete)") {
            Err(DbErr::Custom("[after_delete] Cannot be deleted".to_owned()))
        } else {
            Ok(self)
        }
    }
}

#[test]
fn column_type_test() {
    let _id: sea_orm::NumericColumn<Entity> = COLUMN.id;
    let _name: sea_orm::StringColumn<Entity> = COLUMN.name;
    let _price: sea_orm::NumericColumn<Entity> = COLUMN.price;
    let _bakery_id: sea_orm::NumericColumnNullable<Entity> = COLUMN.bakery_id;
    let _gluten_free: sea_orm::BoolColumn<Entity> = COLUMN.gluten_free;
    let _serial: sea_orm::UuidColumn<Entity> = COLUMN.serial;
}
