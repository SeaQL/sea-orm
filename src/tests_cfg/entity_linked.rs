use crate::entity::prelude::*;

#[derive(Debug)]
pub struct CakeToFilling;

impl Linked for CakeToFilling {
    type FromEntity = super::cake::Entity;

    type ToEntity = super::filling::Entity;

    fn link(&self) -> Vec<LinkDef> {
        vec![
            super::cake_filling::Relation::Cake
                .def()
                .rev()
                .into_link_def()
                .and_on_condition(super::cake::Column::Id.gt(2)),
            super::cake_filling::Relation::Filling.def().into(),
        ]
    }
}

#[derive(Debug)]
pub struct CakeToFillingVendor;

impl Linked for CakeToFillingVendor {
    type FromEntity = super::cake::Entity;

    type ToEntity = super::vendor::Entity;

    fn link(&self) -> Vec<LinkDef> {
        vec![
            super::cake_filling::Relation::Cake.def().rev().into(),
            super::cake_filling::Relation::Filling.def().into(),
            super::filling::Relation::Vendor.def().into(),
        ]
    }
}
