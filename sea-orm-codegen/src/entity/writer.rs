use crate::Entity;
use heck::{SnakeCase, CamelCase};
use sea_orm::{ColumnType, RelationType};
use std::{fs, io::{self, Write}, path::Path, process::Command};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

#[derive(Clone, Debug)]
pub struct EntityWriter {
    pub(crate) entities: Vec<Entity>,
}

impl EntityWriter {
    pub fn generate(self, output_dir: &str) {
        for entity in self.entities.iter() {
            let code_blocks = Self::generate_code(entity);
            Self::write(output_dir, entity, code_blocks).unwrap();
            Self::format(output_dir, entity).unwrap();
        }
    }

    pub fn generate_code(entity: &Entity) -> Vec<TokenStream> {
        let table_name_snake = entity.table_name.to_snake_case();

        let model_field: Vec<Ident> = entity.columns
            .iter()
            .map(|col| {
                format_ident!("{}", col.name.to_snake_case())
            })
            .collect();

        let model_field_type: Vec<Ident> = entity.columns
            .clone()
            .into_iter()
            .map(|col| {
                match col.col_type {
                    ColumnType::Char(_) => format_ident!("String"),
                    ColumnType::String(_) => format_ident!("String"),
                    ColumnType::Text => format_ident!("String"),
                    ColumnType::TinyInteger(_) => format_ident!("u32"),
                    ColumnType::SmallInteger(_) => format_ident!("u32"),
                    ColumnType::Integer(_) => format_ident!("u32"),
                    ColumnType::BigInteger(_) => format_ident!("u32"),
                    ColumnType::Float(_) => format_ident!("f32"),
                    ColumnType::Double(_) => format_ident!("f32"),
                    ColumnType::Decimal(_) => format_ident!("f32"),
                    ColumnType::DateTime(_) => format_ident!("String"),
                    ColumnType::Timestamp(_) => format_ident!("String"),
                    ColumnType::Time(_) => format_ident!("String"),
                    ColumnType::Date => format_ident!("String"),
                    ColumnType::Binary(_) => format_ident!("Vec<u8>"),
                    ColumnType::Boolean => format_ident!("bool"),
                    ColumnType::Money(_) => format_ident!("f32"),
                    ColumnType::Json => format_ident!("String"),
                    ColumnType::JsonBinary => format_ident!("String"),
                    ColumnType::Custom(_) => format_ident!("String"),
                }
            })
            .collect();

        let col_name_camel: Vec<Ident> = entity.columns
            .iter()
            .map(|col| {
                format_ident!("{}", col.name.to_camel_case())
            })
            .collect();

        let primary_key_camel: Vec<Ident> = entity.primary_keys
            .iter()
            .map(|primary_key| {
                format_ident!("{}", primary_key.name.to_camel_case())
            })
            .collect();

        let relation_name_camel: Vec<Ident> = entity.relations
            .iter()
            .map(|rel| {
                format_ident!("{}", rel.ref_table.to_camel_case())
            })
            .collect();

        let relation_name_snake: Vec<Ident> = entity.relations
            .iter()
            .map(|rel| {
                format_ident!("{}", rel.ref_table.to_snake_case())
            })
            .collect();

        let col_type: Vec<TokenStream> = entity.columns
            .clone()
            .into_iter()
            .map(|col| {
                match col.col_type {
                    ColumnType::Char(s) => match s {
                        Some(s) => quote!{ ColumnType::Char(#s) },
                        None => quote!{ ColumnType::Char(None) },
                    },
                    ColumnType::String(s) => match s {
                        Some(s) => quote!{ ColumnType::String(#s) },
                        None => quote!{ ColumnType::String(None) },
                    },
                    ColumnType::Text => quote!{ ColumnType::Text },
                    ColumnType::TinyInteger(s) => match s {
                        Some(s) => quote!{ ColumnType::TinyInteger(#s) },
                        None => quote!{ ColumnType::TinyInteger(None) },
                    },
                    ColumnType::SmallInteger(s) => match s {
                        Some(s) => quote!{ ColumnType::SmallInteger(#s) },
                        None => quote!{ ColumnType::SmallInteger(None) },
                    },
                    ColumnType::Integer(s) => match s {
                        Some(s) => quote!{ ColumnType::Integer(#s) },
                        None => quote!{ ColumnType::Integer(None) },
                    },
                    ColumnType::BigInteger(s) => match s {
                        Some(s) => quote!{ ColumnType::BigInteger(#s) },
                        None => quote!{ ColumnType::BigInteger(None) },
                    },
                    ColumnType::Float(s) => match s {
                        Some(s) => quote!{ ColumnType::Float(#s) },
                        None => quote!{ ColumnType::Float(None) },
                    },
                    ColumnType::Double(s) => match s {
                        Some(s) => quote!{ ColumnType::Double(#s) },
                        None => quote!{ ColumnType::Double(None) },
                    },
                    ColumnType::Decimal(s) => match s {
                        Some((s1, s2)) => quote!{ ColumnType::Decimal((#s1, #s2)) },
                        None => quote!{ ColumnType::Decimal(None) },
                    },
                    ColumnType::DateTime(s) => match s {
                        Some(s) => quote!{ ColumnType::DateTime(#s) },
                        None => quote!{ ColumnType::DateTime(None) },
                    },
                    ColumnType::Timestamp(s) => match s {
                        Some(s) => quote!{ ColumnType::Timestamp(#s) },
                        None => quote!{ ColumnType::Timestamp(None) },
                    },
                    ColumnType::Time(s) => match s {
                        Some(s) => quote!{ ColumnType::Time(#s) },
                        None => quote!{ ColumnType::Time(None) },
                    },
                    ColumnType::Date => quote!{ ColumnType::Date },
                    ColumnType::Binary(s) => match s {
                        Some(s) => quote!{ ColumnType::Binary(#s) },
                        None => quote!{ ColumnType::Binary(None) },
                    },
                    ColumnType::Boolean => quote!{ ColumnType::Boolean },
                    ColumnType::Money(s) => match s {
                        Some((s1, s2)) => quote!{ ColumnType::Money((#s1, #s2)) },
                        None => quote!{ ColumnType::Money(None) },
                    },
                    ColumnType::Json => quote!{ ColumnType::Json },
                    ColumnType::JsonBinary => quote!{ ColumnType::JsonBinary },
                    ColumnType::Custom(s) => {
                        let s = s.to_string();
                        quote!{ ColumnType::Custom(std::rc::Rc::new(sea_query::Alias::new(#s))) }
                    }
                }
            })
            .collect();

        let relation_type: Vec<Ident> = entity.relations
            .iter()
            .map(|rel| {
                match rel.rel_type {
                    RelationType::HasOne => format_ident!("has_one"),
                    RelationType::HasMany => format_ident!("has_many"),
                }
            })
            .collect();

        let relation_col: Vec<Ident> = entity.relations
            .iter()
            .map(|rel| {
                format_ident!("{}", rel.columns[0].to_camel_case())
            })
            .collect();

        let relation_ref_col: Vec<Ident> = entity.relations
            .iter()
            .map(|rel| {
                format_ident!("{}", rel.ref_columns[0].to_camel_case())
            })
            .collect();

        let relation_find_helper: Vec<Ident> = entity.relations
            .iter()
            .map(|rel| {
                format_ident!("find_{}", rel.ref_table.to_snake_case())
            })
            .collect();

        vec![
            quote! {
                use crate as sea_orm;
                use crate::entity::prelude::*;
            },
            quote! {
                #[derive(Copy, Clone, Default, Debug, DeriveEntity)]
                pub struct Entity;
            },
            quote! {
                impl EntityName for Entity {
                    fn table_name(&self) -> &str {
                        #table_name_snake
                    }
                }
            },
            quote! {
                #[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
                pub struct Model {
                    #(pub #model_field: #model_field_type),*
                }
            },
            quote! {
                #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
                pub enum Column {
                    #(#col_name_camel),*
                }
            },
            quote! {
                #[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
                pub enum PrimaryKey {
                    #(#primary_key_camel),*
                }
            },
            quote! {
                #[derive(Copy, Clone, Debug, EnumIter)]
                pub enum Relation {
                    #(#relation_name_camel),*
                }
            },
            quote! {
                impl ColumnTrait for Column {
                    type EntityName = Entity;

                    fn def(&self) -> ColumnType {
                        match self {
                            #(Self::#col_name_camel => #col_type),*
                        }
                    }
                }
            },
            quote! {
                impl RelationTrait for Relation {
                    fn def(&self) -> RelationDef {
                        match self {
                            #(Self::#relation_name_camel => Entity::#relation_type(super::#relation_name_snake::Entity)
                                .from(Column::#relation_col)
                                .to(super::#relation_name_snake::Column::#relation_ref_col)
                                .into()),*
                        }
                    }
                }
            },
            quote! {
                #(impl Related<super::#relation_name_snake::Entity> for Entity {
                    fn to() -> RelationDef {
                        Relation::#relation_name_camel.def()
                    }
                })*
            },
            quote! {
                impl Model {
                    #(pub fn #relation_find_helper(&self) -> Select<super::#relation_name_snake::Entity> {
                        Entity::find_related().belongs_to::<Entity>(self)
                    })*
                }
            },
            quote! {
                impl ActiveModelBehavior for ActiveModel {}
            },
        ]
    }

    pub fn write(output_dir: &str, entity: &Entity, code_blocks: Vec<TokenStream>) -> io::Result<()> {
        let dir = Path::new(output_dir);
        fs::create_dir_all(dir)?;
        let file_path = dir.join(format!("{}.rs", entity.table_name));
        let mut file = fs::File::create(file_path)?;
        for code_block in code_blocks {
            file.write_all(code_block.to_string().as_bytes())?;
            file.write_all(b"\n\n")?;
        }
        Ok(())
    }

    pub fn format(output_dir: &str, entity: &Entity) -> io::Result<()> {
        Command::new("rustfmt")
            .arg(Path::new(output_dir).join(format!("{}.rs", entity.table_name)))
            .spawn()?
            .wait()?;
        Ok(())
    }
}
