use crate as sea_orm;
use crate::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "rust_keyword")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub testing: i32,
    pub rust: i32,
    pub keywords: i32,
    pub r#as: i32,
    pub r#async: i32,
    pub r#await: i32,
    pub r#break: i32,
    pub r#const: i32,
    pub r#continue: i32,
    pub r#dyn: i32,
    pub r#else: i32,
    pub r#enum: i32,
    pub r#extern: i32,
    pub r#false: i32,
    pub r#fn: i32,
    pub r#for: i32,
    pub r#if: i32,
    pub r#impl: i32,
    pub r#in: i32,
    pub r#let: i32,
    pub r#loop: i32,
    pub r#match: i32,
    pub r#mod: i32,
    pub r#move: i32,
    pub r#mut: i32,
    pub r#pub: i32,
    pub r#ref: i32,
    pub r#return: i32,
    pub r#static: i32,
    pub r#struct: i32,
    pub r#trait: i32,
    pub r#true: i32,
    pub r#type: i32,
    pub r#union: i32,
    pub r#unsafe: i32,
    pub r#use: i32,
    pub r#where: i32,
    pub r#while: i32,
    pub r#abstract: i32,
    pub r#become: i32,
    pub r#box: i32,
    pub r#do: i32,
    pub r#final: i32,
    pub r#macro: i32,
    pub r#override: i32,
    pub r#priv: i32,
    pub r#try: i32,
    pub r#typeof: i32,
    pub r#unsized: i32,
    pub r#virtual: i32,
    pub r#yield: i32,
}

#[derive(Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        unreachable!()
    }
}

impl ActiveModelBehavior for ActiveModel {}
