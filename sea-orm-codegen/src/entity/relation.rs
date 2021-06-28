use heck::{CamelCase, SnakeCase};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use sea_query::TableForeignKey;

#[derive(Clone, Debug)]
pub enum RelationType {
    HasOne,
    HasMany,
    BelongsTo,
}

#[derive(Clone, Debug)]
pub struct Relation {
    pub(crate) ref_table: String,
    pub(crate) columns: Vec<String>,
    pub(crate) ref_columns: Vec<String>,
    pub(crate) rel_type: RelationType,
}

impl Relation {
    pub fn get_ref_table_snake_case(&self) -> Ident {
        format_ident!("{}", self.ref_table.to_snake_case())
    }

    pub fn get_ref_table_camel_case(&self) -> Ident {
        format_ident!("{}", self.ref_table.to_camel_case())
    }

    pub fn get_def(&self) -> TokenStream {
        let rel_type = self.get_rel_type();
        let ref_table_snake_case = self.get_ref_table_snake_case();
        match self.rel_type {
            RelationType::HasOne | RelationType::HasMany => {
                quote! {
                    Entity::#rel_type(super::#ref_table_snake_case::Entity).into()
                }
            }
            RelationType::BelongsTo => {
                let column_camel_case = self.get_column_camel_case();
                let ref_column_camel_case = self.get_ref_column_camel_case();
                quote! {
                    Entity::#rel_type(super::#ref_table_snake_case::Entity)
                        .from(Column::#column_camel_case)
                        .to(super::#ref_table_snake_case::Column::#ref_column_camel_case)
                        .into()
                }
            }
        }
    }

    pub fn get_rel_type(&self) -> Ident {
        match self.rel_type {
            RelationType::HasOne => format_ident!("has_one"),
            RelationType::HasMany => format_ident!("has_many"),
            RelationType::BelongsTo => format_ident!("belongs_to"),
        }
    }

    pub fn get_column_camel_case(&self) -> Ident {
        format_ident!("{}", self.columns[0].to_camel_case())
    }

    pub fn get_ref_column_camel_case(&self) -> Ident {
        format_ident!("{}", self.ref_columns[0].to_camel_case())
    }

    pub fn get_rel_find_helper(&self) -> Ident {
        format_ident!("find_{}", self.ref_table.to_snake_case())
    }
}

impl From<&TableForeignKey> for Relation {
    fn from(tbl_fk: &TableForeignKey) -> Self {
        let ref_table = match tbl_fk.get_ref_table() {
            Some(s) => s,
            None => panic!("RefTable should not be empty"),
        };
        let columns = tbl_fk.get_columns();
        let ref_columns = tbl_fk.get_ref_columns();
        let rel_type = RelationType::BelongsTo;
        Self {
            ref_table,
            columns,
            ref_columns,
            rel_type,
        }
    }
}
