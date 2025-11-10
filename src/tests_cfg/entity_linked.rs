use crate::entity::prelude::*;
use sea_query::{Expr, IntoCondition};

#[derive(Debug)]
pub struct CakeToFilling;

impl Linked for CakeToFilling {
    type FromEntity = super::cake::Entity;

    type ToEntity = super::filling::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            super::cake_filling::Relation::Cake.def().rev(),
            super::cake_filling::Relation::Filling.def(),
        ]
    }
}

#[derive(Debug)]
pub struct CakeToFillingVendor;

impl Linked for CakeToFillingVendor {
    type FromEntity = super::cake::Entity;

    type ToEntity = super::vendor::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            super::cake_filling::Relation::Cake.def().rev(),
            super::cake_filling::Relation::Filling.def(),
            super::filling::Relation::Vendor.def(),
        ]
    }
}

#[derive(Debug)]
pub struct CheeseCakeToFillingVendor;

impl Linked for CheeseCakeToFillingVendor {
    type FromEntity = super::cake::Entity;

    type ToEntity = super::vendor::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            super::cake_filling::Relation::Cake
                .def()
                .on_condition(|left, _right| {
                    Expr::col((left, super::cake::Column::Name))
                        .like("%cheese%")
                        .into_condition()
                })
                .rev(),
            super::cake_filling::Relation::Filling.def(),
            super::filling::Relation::Vendor.def(),
        ]
    }
}

#[derive(Debug)]
pub struct CakeToCakeViaFilling;

impl Linked for CakeToCakeViaFilling {
    type FromEntity = super::cake::Entity;

    type ToEntity = super::cake::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            super::cake_filling::Relation::Cake.def().rev(),
            super::cake_filling::Relation::Filling.def(),
            super::cake_filling::Relation::Filling.def().rev(),
            super::cake_filling::Relation::Cake.def(),
        ]
    }
}
