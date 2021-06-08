use crate::Entity;
use std::{fs, io::{self, Write}, path::Path, process::Command};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Clone, Debug)]
pub struct EntityWriter {
    pub(crate) entities: Vec<Entity>,
}

impl EntityWriter {
    pub fn generate(self, output_dir: &str) {
        for entity in self.entities.iter() {
            let code_blocks = Self::gen_code_blocks(entity);
            Self::write(output_dir, entity, code_blocks).unwrap();
            Self::format(output_dir, entity).unwrap();
        }
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

    pub fn gen_code_blocks(entity: &Entity) -> Vec<TokenStream> {
        vec![
            Self::gen_import(),
            Self::gen_entity_struct(),
            Self::gen_impl_entity_name(entity),
            Self::gen_model_struct(entity),
            Self::gen_column_enum(entity),
            Self::gen_primary_key_enum(entity),
            Self::gen_relation_enum(entity),
            Self::gen_impl_column_trait(entity),
            Self::gen_impl_relation_trait(entity),
            Self::gen_impl_related(entity),
            Self::gen_impl_model(entity),
            Self::gen_impl_active_model_behavior(),
        ]
    }

    pub fn gen_import() -> TokenStream {
        quote! {
            use crate as sea_orm;
            use crate::entity::prelude::*;
        }
    }

    pub fn gen_entity_struct() -> TokenStream {
        quote! {
            #[derive(Copy, Clone, Default, Debug, DeriveEntity)]
            pub struct Entity;
        }
    }

    pub fn gen_impl_entity_name(entity: &Entity) -> TokenStream {
        let table_name_snake_case = entity.get_table_name_snake_case();
        quote! {
            impl EntityName for Entity {
                fn table_name(&self) -> &str {
                    #table_name_snake_case
                }
            }
        }
    }

    pub fn gen_model_struct(entity: &Entity) -> TokenStream {
        let column_names_snake_case = entity.get_column_names_snake_case();
        let column_rs_types = entity.get_column_rs_types();
        quote! {
            #[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
            pub struct Model {
                #(pub #column_names_snake_case: #column_rs_types),*
            }
        }
    }

    pub fn gen_column_enum(entity: &Entity) -> TokenStream {
        let column_names_camel_case = entity.get_column_names_camel_case();
        quote! {
            #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
            pub enum Column {
                #(#column_names_camel_case),*
            }
        }
    }

    pub fn gen_primary_key_enum(entity: &Entity) -> TokenStream {
        let primary_key_names_camel_case = entity.get_primary_key_names_camel_case();
        quote! {
            #[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
            pub enum PrimaryKey {
                #(#primary_key_names_camel_case),*
            }
        }
    }

    pub fn gen_relation_enum(entity: &Entity) -> TokenStream {
        let relation_ref_tables_camel_case = entity.get_relation_ref_tables_camel_case();
        quote! {
            #[derive(Copy, Clone, Debug, EnumIter)]
            pub enum Relation {
                #(#relation_ref_tables_camel_case),*
            }
        }
    }

    pub fn gen_impl_column_trait(entity: &Entity) -> TokenStream {
        let column_names_camel_case = entity.get_column_names_camel_case();
        let column_types = entity.get_column_types();
        quote! {
            impl ColumnTrait for Column {
                type EntityName = Entity;

                fn def(&self) -> ColumnType {
                    match self {
                        #(Self::#column_names_camel_case => #column_types),*
                    }
                }
            }
        }
    }

    pub fn gen_impl_relation_trait(entity: &Entity) -> TokenStream {
        let relation_ref_tables_camel_case = entity.get_relation_ref_tables_camel_case();
        let relation_rel_types = entity.get_relation_rel_types();
        let relation_ref_tables_snake_case = entity.get_relation_ref_tables_snake_case();
        let relation_columns_camel_case = entity.get_relation_columns_camel_case();
        let relation_ref_columns_camel_case = entity.get_relation_ref_columns_camel_case();
        quote! {
            impl RelationTrait for Relation {
                fn def(&self) -> RelationDef {
                    match self {
                        #(Self::#relation_ref_tables_camel_case => Entity::#relation_rel_types(super::#relation_ref_tables_snake_case::Entity)
                            .from(Column::#relation_columns_camel_case)
                            .to(super::#relation_ref_tables_snake_case::Column::#relation_ref_columns_camel_case)
                            .into()),*
                    }
                }
            }
        }
    }

    pub fn gen_impl_related(entity: &Entity) -> TokenStream {
        let relation_ref_tables_camel_case = entity.get_relation_ref_tables_camel_case();
        let relation_ref_tables_snake_case = entity.get_relation_ref_tables_snake_case();
        quote! {
            #(impl Related<super::#relation_ref_tables_snake_case::Entity> for Entity {
                fn to() -> RelationDef {
                    Relation::#relation_ref_tables_camel_case.def()
                }
            })*
        }
    }

    pub fn gen_impl_model(entity: &Entity) -> TokenStream {
        let relation_ref_tables_snake_case = entity.get_relation_ref_tables_snake_case();
        let relation_rel_find_helpers = entity.get_relation_rel_find_helpers();
        quote! {
            impl Model {
                #(pub fn #relation_rel_find_helpers(&self) -> Select<super::#relation_ref_tables_snake_case::Entity> {
                    Entity::find_related().belongs_to::<Entity>(self)
                })*
            }
        }
    }

    pub fn gen_impl_active_model_behavior() -> TokenStream {
        quote! {
            impl ActiveModelBehavior for ActiveModel {}
        }
    }
}
