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
    #[sea_orm(related = "Bakery")]
    pub bakery: BelongsTo<super::bakery::Entity>,
    #[sea_orm(related = "Baker", via = "cakes_bakers::Cake")]
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
pub struct EntityLoader {
    select: sea_orm::Select<Entity>,
    with_bakery: bool,
    with_baker: bool,
}

impl sea_orm::QueryFilter for EntityLoader {
    type QueryStatement = <sea_orm::Select<Entity> as sea_orm::QueryFilter>::QueryStatement;

    fn query(&mut self) -> &mut Self::QueryStatement {
        sea_orm::QueryFilter::query(&mut self.select)
    }
}

impl sea_orm::QueryOrder for EntityLoader {
    type QueryStatement = <sea_orm::Select<Entity> as sea_orm::QueryOrder>::QueryStatement;

    fn query(&mut self) -> &mut Self::QueryStatement {
        sea_orm::QueryOrder::query(&mut self.select)
    }
}

impl Entity {
    pub fn loader() -> EntityLoader {
        EntityLoader {
            select: Entity::find(),
            with_bakery: false,
            with_baker: false,
        }
    }
}

impl EntityLoader {
    pub fn with<E: EntityTrait>(mut self, entity: E) -> Self {
        if entity.table_ref() == super::bakery::Entity.table_ref() {
            self.with_bakery = true;
        }
        if entity.table_ref() == super::baker::Entity.table_ref() {
            self.with_baker = true;
        }
        self
    }

    pub async fn one<C: sea_orm::ConnectionTrait>(
        mut self,
        db: &C,
    ) -> Result<Option<Model>, DbErr> {
        use sea_orm::QuerySelect;

        self.select = self.select.limit(1);
        Ok(self.all(db).await?.into_iter().next())
    }

    pub async fn all<C: sea_orm::ConnectionTrait>(self, db: &C) -> Result<Vec<Model>, DbErr> {
        let select = if self.with_bakery {
            self.select.find_also(Entity, super::bakery::Entity)
        } else {
            // select also but without join
            self.select.select_also_fake(super::bakery::Entity)
        };

        let models = select.all(db).await?;

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
