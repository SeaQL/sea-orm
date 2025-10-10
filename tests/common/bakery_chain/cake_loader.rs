use sea_orm::{ConnectionTrait, compound::*, entity::prelude::*};

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
    #[sea_orm(ignore)]
    pub bakery: BelongsTo<super::bakery::Entity>,
    #[sea_orm(ignore)]
    pub bakers: HasMany<super::baker::Entity>,
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

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        use sea_orm::Set;
        Self {
            serial: Set(Uuid::new_v4()),
            ..ActiveModelTrait::default()
        }
    }
}

// intended to be generated
#[derive(Default)]
pub struct EntityLoader {
    with_bakery: bool,
    with_baker: bool,
}

impl Entity {
    pub fn loader() -> EntityLoader {
        Default::default()
    }
}

impl EntityLoader {
    pub fn with<E: EntityTrait>(mut self, entity: E) -> Self {
        if entity.table_ref() == super::bakery::Entity.table_ref() {
            self.with_bakery = true;
        }
        if entity.table_ref() == super::bakery::Entity.table_ref() {
            self.with_baker = true;
        }
        self
    }

    pub async fn all<C: sea_orm::ConnectionTrait>(self, db: &C) -> Result<Vec<Model>, DbErr> {
        let query = Entity::find();
        let query = if self.with_bakery {
            query.find_also(Entity, super::bakery::Entity)
        } else {
            // select also but without join
            query.select_also_fake(super::bakery::Entity)
        };

        let models = query.all(db).await?;

        let mut cakes = Vec::new();

        for (mut cake, bakery) in models {
            cake.bakery.set(bakery.map(Box::new));
            cakes.push(cake);
        }

        if self.with_baker {
            let bakers = cakes.load_many(super::baker::Entity, db).await?;

            for (cake, bakers) in cakes.iter_mut().zip(bakers) {
                cake.bakers.set(bakers)
            }
        }

        Ok(cakes)
    }
}
