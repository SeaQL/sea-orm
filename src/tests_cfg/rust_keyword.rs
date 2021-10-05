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
    pub r#raw_identifier: i32,
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

#[cfg(test)]
mod tests {
    use crate::tests_cfg::*;
    use sea_query::Iden;

    #[test]
    fn test_columns() {
        assert_eq!(rust_keyword::Column::Id.to_string(), "id".to_owned());
        assert_eq!(rust_keyword::Column::Testing.to_string(), "testing".to_owned());
        assert_eq!(rust_keyword::Column::Rust.to_string(), "rust".to_owned());
        assert_eq!(rust_keyword::Column::Keywords.to_string(), "keywords".to_owned());
        assert_eq!(rust_keyword::Column::RawIdentifier.to_string(), "raw_identifier".to_owned());
        assert_eq!(rust_keyword::Column::As.to_string(), "as".to_owned());
        assert_eq!(rust_keyword::Column::Async.to_string(), "async".to_owned());
        assert_eq!(rust_keyword::Column::Await.to_string(), "await".to_owned());
        assert_eq!(rust_keyword::Column::Break.to_string(), "break".to_owned());
        assert_eq!(rust_keyword::Column::Const.to_string(), "const".to_owned());
        assert_eq!(rust_keyword::Column::Continue.to_string(), "continue".to_owned());
        assert_eq!(rust_keyword::Column::Dyn.to_string(), "dyn".to_owned());
        assert_eq!(rust_keyword::Column::Else.to_string(), "else".to_owned());
        assert_eq!(rust_keyword::Column::Enum.to_string(), "enum".to_owned());
        assert_eq!(rust_keyword::Column::Extern.to_string(), "extern".to_owned());
        assert_eq!(rust_keyword::Column::False.to_string(), "false".to_owned());
        assert_eq!(rust_keyword::Column::Fn.to_string(), "fn".to_owned());
        assert_eq!(rust_keyword::Column::For.to_string(), "for".to_owned());
        assert_eq!(rust_keyword::Column::If.to_string(), "if".to_owned());
        assert_eq!(rust_keyword::Column::Impl.to_string(), "impl".to_owned());
        assert_eq!(rust_keyword::Column::In.to_string(), "in".to_owned());
        assert_eq!(rust_keyword::Column::Let.to_string(), "let".to_owned());
        assert_eq!(rust_keyword::Column::Loop.to_string(), "loop".to_owned());
        assert_eq!(rust_keyword::Column::Match.to_string(), "match".to_owned());
        assert_eq!(rust_keyword::Column::Mod.to_string(), "mod".to_owned());
        assert_eq!(rust_keyword::Column::Move.to_string(), "move".to_owned());
        assert_eq!(rust_keyword::Column::Mut.to_string(), "mut".to_owned());
        assert_eq!(rust_keyword::Column::Pub.to_string(), "pub".to_owned());
        assert_eq!(rust_keyword::Column::Ref.to_string(), "ref".to_owned());
        assert_eq!(rust_keyword::Column::Return.to_string(), "return".to_owned());
        assert_eq!(rust_keyword::Column::Static.to_string(), "static".to_owned());
        assert_eq!(rust_keyword::Column::Struct.to_string(), "struct".to_owned());
        assert_eq!(rust_keyword::Column::Trait.to_string(), "trait".to_owned());
        assert_eq!(rust_keyword::Column::True.to_string(), "true".to_owned());
        assert_eq!(rust_keyword::Column::Type.to_string(), "type".to_owned());
        assert_eq!(rust_keyword::Column::Union.to_string(), "union".to_owned());
        assert_eq!(rust_keyword::Column::Unsafe.to_string(), "unsafe".to_owned());
        assert_eq!(rust_keyword::Column::Use.to_string(), "use".to_owned());
        assert_eq!(rust_keyword::Column::Where.to_string(), "where".to_owned());
        assert_eq!(rust_keyword::Column::While.to_string(), "while".to_owned());
        assert_eq!(rust_keyword::Column::Abstract.to_string(), "abstract".to_owned());
        assert_eq!(rust_keyword::Column::Become.to_string(), "become".to_owned());
        assert_eq!(rust_keyword::Column::Box.to_string(), "box".to_owned());
        assert_eq!(rust_keyword::Column::Do.to_string(), "do".to_owned());
        assert_eq!(rust_keyword::Column::Final.to_string(), "final".to_owned());
        assert_eq!(rust_keyword::Column::Macro.to_string(), "macro".to_owned());
        assert_eq!(rust_keyword::Column::Override.to_string(), "override".to_owned());
        assert_eq!(rust_keyword::Column::Priv.to_string(), "priv".to_owned());
        assert_eq!(rust_keyword::Column::Try.to_string(), "try".to_owned());
        assert_eq!(rust_keyword::Column::Typeof.to_string(), "typeof".to_owned());
        assert_eq!(rust_keyword::Column::Unsized.to_string(), "unsized".to_owned());
        assert_eq!(rust_keyword::Column::Virtual.to_string(), "virtual".to_owned());
        assert_eq!(rust_keyword::Column::Yield.to_string(), "yield".to_owned());
    }
}
