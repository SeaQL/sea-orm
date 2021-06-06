use crate::{Column, Entity, Relation};
use heck::{SnakeCase, CamelCase};
use sea_orm::{ColumnType, RelationType};
use sea_query::{ColumnSpec, TableStatement};
use sea_schema::mysql::{def::Schema, discovery::SchemaDiscovery};
use sqlx::MySqlPool;
use syn::{Fields, Variant, parse_quote};
use std::{collections::HashMap, fs, io::{self, Write}, mem::swap, path::Path, process::Command};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

#[derive(Clone, Debug)]
pub struct EntityGenerator {
    pub(crate) entities: Vec<Entity>,
    pub(crate) inverse_relations: HashMap<String, Vec<Relation>>,
}

impl EntityGenerator {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            inverse_relations: HashMap::new(),
        }
    }

    pub async fn discover(uri: &str, schema: &str) -> Schema {
        let connection = MySqlPool::connect(uri).await.unwrap();
        let schema_discovery = SchemaDiscovery::new(connection, schema);
        schema_discovery.discover().await
    }

    pub async fn parse(mut self, uri: &str, schema: &str) -> Self {
        let schema = Self::discover(uri, schema).await;
        for table_ref in schema.tables.iter() {
            let table_stmt = table_ref.write();
            // TODO: why return TableStatement?
            let table_create = match table_stmt {
                TableStatement::Create(stmt) => stmt,
                _ => panic!("TableStatement should be create"),
            };
            println!("{:#?}", table_create);
            let table_name = match table_create.get_table_name() {
                Some(s) => s,
                None => panic!("Table name should not be empty"),
            };
            let columns = table_create.get_columns()
                .iter()
                .map(|col| {
                    let name = col.get_column_name();
                    let rs_type = "some_rust_type".to_string();
                    let col_type = match col.get_column_type() {
                        Some(ty) => ty.clone(),
                        None => panic!("ColumnType should not be empty"),
                    };
                    let is_primary_key = col.get_column_spec()
                        .iter()
                        .find(|s| {
                            match s {
                                ColumnSpec::PrimaryKey => true,
                                _ => false,
                            }
                        })
                        .is_some();
                    Column {
                        name,
                        rs_type,
                        col_type,
                        is_primary_key,
                    }
                })
                .collect();
            let relations = table_create.get_foreign_key_create_stmts()
                .iter()
                .map(|fk_stmt| fk_stmt.get_foreign_key())
                .map(|fk| {
                    let ref_table = match fk.get_ref_table() {
                        Some(s) => s,
                        None => panic!("RefTable should not be empty"),
                    };
                    let columns = fk.get_columns();
                    let ref_columns = fk.get_ref_columns();
                    let rel_type = RelationType::HasOne;
                    Relation {
                        ref_table,
                        columns,
                        ref_columns,
                        rel_type,
                    }
                });
            self.entities.push(Entity {
                table_name: table_name.clone(),
                columns,
                relations: relations.clone().collect(),
            });
            for mut rel in relations.into_iter() {
                let ref_table = rel.ref_table;
                swap(&mut rel.columns, &mut rel.ref_columns);
                rel.rel_type = RelationType::HasMany;
                rel.ref_table = table_name.clone();
                if let Some(vec) = self.inverse_relations.get_mut(&ref_table) {
                    vec.push(rel);
                } else {
                    self.inverse_relations.insert(ref_table, vec![rel]);
                }
            }
        }
        for (tbl_name, relations) in self.inverse_relations.iter() {
            for ent in self.entities.iter_mut() {
                if ent.table_name.eq(tbl_name) {
                    ent.relations.append(relations.clone().as_mut());
                }
            }
        }
        println!();
        println!("entities:");
        println!("{:#?}", self.entities);
        println!();
        println!("inverse_relations:");
        println!("{:#?}", self.inverse_relations);
        self
    }

    pub fn write(self, path: &str) -> io::Result<Self> {
        let dir_path = Path::new(path);
        fs::create_dir_all(dir_path)?;
        for entity in self.entities.iter() {
            let file_path = dir_path.join(format!("{}.rs", entity.table_name));
            let mut file = fs::File::create(file_path)?;
            for code_block in Self::generate_code(entity) {
                file.write_all(code_block.to_string().as_bytes())?;
                file.write_all(b"\n\n")?;
            }
        }
        self.format(path)
    }

    pub fn format(self, path: &str) -> io::Result<Self> {
        for entity in self.entities.iter() {
            Command::new("rustfmt")
                .arg(Path::new(path).join(format!("{}.rs", entity.table_name)))
                .spawn()?
                .wait()?;
        }
        Ok(self)
    }

    pub fn generate_code(entity: &Entity) -> Vec<TokenStream> {
        let table_name_snake = entity.table_name.to_snake_case();
        let table_name_camel = entity.table_name.to_camel_case();

        let model_field: Vec<Ident> = entity.columns
            .iter()
            .map(|col| {
                format_ident!("{}", col.name.to_snake_case())
            })
            .collect();

        let model_field_type: Vec<Ident> = entity.columns
            .iter()
            .map(|col| {
                format_ident!("{}", col.rs_type)
            })
            .collect();

        let col_name_camel: Vec<Ident> = entity.columns
            .iter()
            .map(|col| {
                format_ident!("{}", col.name.to_camel_case())
            })
            .collect();

        let primary_key_camel: Vec<Ident> = entity.columns
            .iter()
            .filter(|col| col.is_primary_key)
            .map(|col| {
                format_ident!("{}", col.name.to_camel_case())
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
                    ColumnType::Char(s) => quote!{ ColumnType::Char(s) },
                    ColumnType::String(s) => quote!{ ColumnType::String(s) },
                    ColumnType::Text => quote!{ ColumnType::Text },
                    ColumnType::TinyInteger(s) => quote!{ ColumnType::TinyInteger(s) },
                    ColumnType::SmallInteger(s) => quote!{ ColumnType::SmallInteger(s) },
                    ColumnType::Integer(s) => quote!{ ColumnType::Integer(s) },
                    ColumnType::BigInteger(s) => quote!{ ColumnType::BigInteger(s) },
                    ColumnType::Float(s) => quote!{ ColumnType::Float(s) },
                    ColumnType::Double(s) => quote!{ ColumnType::Double(s) },
                    ColumnType::Decimal(s) => quote!{ ColumnType::Decimal(s) },
                    ColumnType::DateTime(s) => quote!{ ColumnType::DateTime(s) },
                    ColumnType::Timestamp(s) => quote!{ ColumnType::Timestamp(s) },
                    ColumnType::Time(s) => quote!{ ColumnType::Time(s) },
                    ColumnType::Date => quote!{ ColumnType::Date },
                    ColumnType::Binary(s) => quote!{ ColumnType::Binary(s) },
                    ColumnType::Boolean => quote!{ ColumnType::Boolean },
                    ColumnType::Money(s) => quote!{ ColumnType::Money(s) },
                    ColumnType::Json => quote!{ ColumnType::Json },
                    ColumnType::JsonBinary => quote!{ ColumnType::JsonBinary },
                    ColumnType::Custom(s) => quote!{ ColumnType::Custom(s) },
                }
            })
            .collect();

        let relation_type: Vec<Ident> = entity.relations
            .iter()
            .map(|rel| {
                match rel.rel_type {
                    RelationType::HasOne => format_ident!("has_one"),
                    RelationType::HasMany => format_ident!("has_Many"),
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
}
