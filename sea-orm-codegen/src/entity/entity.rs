use crate::{Column, PrimaryKey, Relation};
use heck::SnakeCase;
use proc_macro2::{Ident, TokenStream};

#[derive(Clone, Debug)]
pub struct Entity {
    pub(crate) table_name: String,
    pub(crate) columns: Vec<Column>,
    pub(crate) relations: Vec<Relation>,
    pub(crate) primary_keys: Vec<PrimaryKey>,
}

impl Entity {
    pub fn get_table_name_snake_case(&self) -> String {
        self.table_name.to_snake_case()
    }

    pub fn get_column_names_snake_case(&self) -> Vec<Ident> {
        self.columns
            .iter()
            .map(|col| col.get_name_snake_case())
            .collect()
    }

    pub fn get_column_names_camel_case(&self) -> Vec<Ident> {
        self.columns
            .iter()
            .map(|col| col.get_name_camel_case())
            .collect()
    }

    pub fn get_column_rs_types(&self) -> Vec<Ident> {
        self.columns
            .clone()
            .into_iter()
            .map(|col| col.get_rs_type())
            .collect()
    }

    pub fn get_column_types(&self) -> Vec<TokenStream> {
        self.columns
            .clone()
            .into_iter()
            .map(|col| col.get_type())
            .collect()
    }

    pub fn get_primary_key_names_snake_case(&self) -> Vec<Ident> {
        self.primary_keys
            .iter()
            .map(|pk| pk.get_name_snake_case())
            .collect()
    }

    pub fn get_primary_key_names_camel_case(&self) -> Vec<Ident> {
        self.primary_keys
            .iter()
            .map(|pk| pk.get_name_camel_case())
            .collect()
    }

    pub fn get_relation_ref_tables_snake_case(&self) -> Vec<Ident> {
        self.relations
            .iter()
            .map(|rel| rel.get_ref_table_snake_case())
            .collect()
    }

    pub fn get_relation_ref_tables_camel_case(&self) -> Vec<Ident> {
        self.relations
            .iter()
            .map(|rel| rel.get_ref_table_camel_case())
            .collect()
    }

    pub fn get_relation_rel_types(&self) -> Vec<Ident> {
        self.relations
            .iter()
            .map(|rel| rel.get_rel_type())
            .collect()
    }

    pub fn get_relation_columns_camel_case(&self) -> Vec<Ident> {
        self.relations
            .iter()
            .map(|rel| rel.get_column_camel_case())
            .collect()
    }

    pub fn get_relation_ref_columns_camel_case(&self) -> Vec<Ident> {
        self.relations
            .iter()
            .map(|rel| rel.get_ref_column_camel_case())
            .collect()
    }

    pub fn get_relation_rel_find_helpers(&self) -> Vec<Ident> {
        self.relations
            .iter()
            .map(|rel| rel.get_rel_find_helper())
            .collect()
    }
}
