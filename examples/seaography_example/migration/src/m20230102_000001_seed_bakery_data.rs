use crate::entity::{prelude::*, *};
use sea_orm::entity::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let bakery = bakery::ActiveModel {
            name: Set("SeaSide Bakery".to_owned()),
            profit_margin: Set(10.4),
            ..Default::default()
        };
        let sea = Bakery::insert(bakery).exec(db).await?.last_insert_id;

        let bakery = bakery::ActiveModel {
            name: Set("LakeSide Bakery".to_owned()),
            profit_margin: Set(5.8),
            ..Default::default()
        };
        let lake = Bakery::insert(bakery).exec(db).await?.last_insert_id;

        let alice = baker::ActiveModel {
            name: Set("Alice".to_owned()),
            contact: Set("+44 15273388".to_owned()),
            bakery_id: Set(Some(sea)),
            ..Default::default()
        };
        let alice = Baker::insert(alice).exec(db).await?.last_insert_id;

        let bob = baker::ActiveModel {
            name: Set("Bob".to_owned()),
            contact: Set("+852 12345678".to_owned()),
            bakery_id: Set(Some(lake)),
            ..Default::default()
        };
        let bob = Baker::insert(bob).exec(db).await?.last_insert_id;

        let cake = cake::ActiveModel {
            name: Set("Chocolate Cake".to_owned()),
            price: Set("10.25".parse().unwrap()),
            gluten_free: Set(0),
            bakery_id: Set(Some(sea)),
            ..Default::default()
        };
        let choco = Cake::insert(cake).exec(db).await?.last_insert_id;

        let mut cake = cake::ActiveModel {
            name: Set("Double Chocolate".to_owned()),
            price: Set("12.5".parse().unwrap()),
            gluten_free: Set(0),
            bakery_id: Set(Some(sea)),
            ..Default::default()
        };
        let double_1 = Cake::insert(cake.clone()).exec(db).await?.last_insert_id;
        cake.bakery_id = Set(Some(lake));
        let double_2 = Cake::insert(cake).exec(db).await?.last_insert_id;

        let mut cake = cake::ActiveModel {
            name: Set("Lemon Cake".to_owned()),
            price: Set("8.8".parse().unwrap()),
            gluten_free: Set(0),
            bakery_id: Set(Some(sea)),
            ..Default::default()
        };
        let lemon_1 = Cake::insert(cake.clone()).exec(db).await?.last_insert_id;
        cake.bakery_id = Set(Some(lake));
        let lemon_2 = Cake::insert(cake).exec(db).await?.last_insert_id;

        let mut cake = cake::ActiveModel {
            name: Set("Strawberry Cake".to_owned()),
            price: Set("9.9".parse().unwrap()),
            gluten_free: Set(0),
            bakery_id: Set(Some(sea)),
            ..Default::default()
        };
        let straw_1 = Cake::insert(cake.clone()).exec(db).await?.last_insert_id;
        cake.bakery_id = Set(Some(lake));
        let straw_2 = Cake::insert(cake).exec(db).await?.last_insert_id;

        let cake = cake::ActiveModel {
            name: Set("Orange Cake".to_owned()),
            price: Set("6.5".parse().unwrap()),
            gluten_free: Set(1),
            bakery_id: Set(Some(lake)),
            ..Default::default()
        };
        let orange = Cake::insert(cake).exec(db).await?.last_insert_id;

        let mut cake = cake::ActiveModel {
            name: Set("New York Cheese".to_owned()),
            price: Set("12.5".parse().unwrap()),
            gluten_free: Set(0),
            bakery_id: Set(Some(sea)),
            ..Default::default()
        };
        let cheese_1 = Cake::insert(cake.clone()).exec(db).await?.last_insert_id;
        cake.bakery_id = Set(Some(lake));
        let cheese_2 = Cake::insert(cake).exec(db).await?.last_insert_id;

        let mut cake = cake::ActiveModel {
            name: Set("Blueburry Cheese".to_owned()),
            price: Set("11.5".parse().unwrap()),
            gluten_free: Set(1),
            bakery_id: Set(Some(sea)),
            ..Default::default()
        };
        let blue_1 = Cake::insert(cake.clone()).exec(db).await?.last_insert_id;
        cake.bakery_id = Set(Some(lake));
        let blue_2 = Cake::insert(cake).exec(db).await?.last_insert_id;

        let rel = cake_baker::ActiveModel {
            cake_id: Set(choco),
            baker_id: Set(alice),
        };
        CakeBaker::insert(rel).exec(db).await?;

        let rel = cake_baker::ActiveModel {
            cake_id: Set(double_1),
            baker_id: Set(alice),
        };
        CakeBaker::insert(rel).exec(db).await?;
        let rel = cake_baker::ActiveModel {
            cake_id: Set(double_2),
            baker_id: Set(bob),
        };
        CakeBaker::insert(rel).exec(db).await?;

        let rel = cake_baker::ActiveModel {
            cake_id: Set(lemon_1),
            baker_id: Set(alice),
        };
        CakeBaker::insert(rel).exec(db).await?;
        let rel = cake_baker::ActiveModel {
            cake_id: Set(lemon_2),
            baker_id: Set(bob),
        };
        CakeBaker::insert(rel).exec(db).await?;

        let rel = cake_baker::ActiveModel {
            cake_id: Set(straw_1),
            baker_id: Set(alice),
        };
        CakeBaker::insert(rel).exec(db).await?;
        let rel = cake_baker::ActiveModel {
            cake_id: Set(straw_2),
            baker_id: Set(bob),
        };
        CakeBaker::insert(rel).exec(db).await?;

        let rel = cake_baker::ActiveModel {
            cake_id: Set(orange),
            baker_id: Set(bob),
        };
        CakeBaker::insert(rel).exec(db).await?;

        let rel = cake_baker::ActiveModel {
            cake_id: Set(cheese_1),
            baker_id: Set(alice),
        };
        CakeBaker::insert(rel).exec(db).await?;
        let rel = cake_baker::ActiveModel {
            cake_id: Set(cheese_2),
            baker_id: Set(bob),
        };
        CakeBaker::insert(rel).exec(db).await?;

        let rel = cake_baker::ActiveModel {
            cake_id: Set(blue_1),
            baker_id: Set(alice),
        };
        CakeBaker::insert(rel).exec(db).await?;
        let rel = cake_baker::ActiveModel {
            cake_id: Set(blue_2),
            baker_id: Set(bob),
        };
        CakeBaker::insert(rel).exec(db).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        Cake::delete_many().exec(db).await?;
        Baker::delete_many().exec(db).await?;
        Bakery::delete_many().exec(db).await?;

        Ok(())
    }
}
