use sea_orm::RelationType;
use sea_query::TableForeignKey;
use heck::{SnakeCase, CamelCase};
use proc_macro2::Ident;
use quote::format_ident;

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

    pub fn get_rel_type(&self) -> Ident {
        match self.rel_type {
            RelationType::HasOne => format_ident!("has_one"),
            RelationType::HasMany => format_ident!("has_many"),
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
        let rel_type = RelationType::HasOne;
        Self {
            ref_table,
            columns,
            ref_columns,
            rel_type,
        }
    }
}
