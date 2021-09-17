use heck::{CamelCase, SnakeCase};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use sea_query::{ForeignKeyAction, TableForeignKey};

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
    pub(crate) on_update: Option<ForeignKeyAction>,
    pub(crate) on_delete: Option<ForeignKeyAction>,
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

    pub fn get_attrs(&self) -> TokenStream {
        let rel_type = self.get_rel_type();
        let ref_table_snake_case = self.get_ref_table_snake_case();
        let ref_entity = format!("super::{}::Entity", ref_table_snake_case);
        match self.rel_type {
            RelationType::HasOne | RelationType::HasMany => {
                quote! {
                    #[sea_orm(#rel_type = #ref_entity)]
                }
            }
            RelationType::BelongsTo => {
                let column_camel_case = self.get_column_camel_case();
                let ref_column_camel_case = self.get_ref_column_camel_case();
                let from = format!("Column::{}", column_camel_case);
                let to = format!(
                    "super::{}::Column::{}",
                    ref_table_snake_case, ref_column_camel_case
                );
                let on_update = if let Some(action) = &self.on_update {
                    let action = Self::get_foreign_key_action(action);
                    quote! {
                        on_update = #action,
                    }
                } else {
                    TokenStream::new()
                };
                let on_delete = if let Some(action) = &self.on_delete {
                    let action = Self::get_foreign_key_action(action);
                    quote! {
                        on_delete = #action,
                    }
                } else {
                    TokenStream::new()
                };
                quote! {
                    #[sea_orm(
                        #rel_type = #ref_entity,
                        from = #from,
                        to = #to,
                        #on_update
                        #on_delete
                    )]
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

    pub fn get_foreign_key_action(action: &ForeignKeyAction) -> String {
        match action {
            ForeignKeyAction::Restrict => "Restrict",
            ForeignKeyAction::Cascade => "Cascade",
            ForeignKeyAction::SetNull => "SetNull",
            ForeignKeyAction::NoAction => "NoAction",
            ForeignKeyAction::SetDefault => "SetDefault",
        }
        .to_owned()
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
        let on_delete = tbl_fk.get_on_delete();
        let on_update = tbl_fk.get_on_update();
        Self {
            ref_table,
            columns,
            ref_columns,
            rel_type,
            on_delete,
            on_update,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Relation, RelationType};
    use proc_macro2::TokenStream;
    use sea_query::ForeignKeyAction;

    fn setup() -> Vec<Relation> {
        vec![
            Relation {
                ref_table: "fruit".to_owned(),
                columns: vec!["id".to_owned()],
                ref_columns: vec!["cake_id".to_owned()],
                rel_type: RelationType::HasOne,
                on_delete: None,
                on_update: None,
            },
            Relation {
                ref_table: "filling".to_owned(),
                columns: vec!["filling_id".to_owned()],
                ref_columns: vec!["id".to_owned()],
                rel_type: RelationType::BelongsTo,
                on_delete: Some(ForeignKeyAction::Cascade),
                on_update: Some(ForeignKeyAction::Cascade),
            },
            Relation {
                ref_table: "filling".to_owned(),
                columns: vec!["filling_id".to_owned()],
                ref_columns: vec!["id".to_owned()],
                rel_type: RelationType::HasMany,
                on_delete: Some(ForeignKeyAction::Cascade),
                on_update: None,
            },
        ]
    }

    #[test]
    fn test_get_ref_table_snake_case() {
        let relations = setup();
        let snake_cases = vec!["fruit", "filling", "filling"];
        for (rel, snake_case) in relations.into_iter().zip(snake_cases) {
            assert_eq!(rel.get_ref_table_snake_case().to_string(), snake_case);
        }
    }

    #[test]
    fn test_get_ref_table_camel_case() {
        let relations = setup();
        let camel_cases = vec!["Fruit", "Filling", "Filling"];
        for (rel, camel_case) in relations.into_iter().zip(camel_cases) {
            assert_eq!(rel.get_ref_table_camel_case().to_string(), camel_case);
        }
    }

    #[test]
    fn test_get_def() {
        let relations = setup();
        let rel_defs = vec![
            "Entity::has_one(super::fruit::Entity).into()",
            "Entity::belongs_to(super::filling::Entity) \
                .from(Column::FillingId) \
                .to(super::filling::Column::Id) \
                .into()",
            "Entity::has_many(super::filling::Entity).into()",
        ];
        for (rel, rel_def) in relations.into_iter().zip(rel_defs) {
            let rel_def: TokenStream = rel_def.parse().unwrap();

            assert_eq!(rel.get_def().to_string(), rel_def.to_string());
        }
    }

    #[test]
    fn test_get_rel_type() {
        let relations = setup();
        let rel_types = vec!["has_one", "belongs_to", "has_many"];
        for (rel, rel_type) in relations.into_iter().zip(rel_types) {
            assert_eq!(rel.get_rel_type(), rel_type);
        }
    }

    #[test]
    fn test_get_column_camel_case() {
        let relations = setup();
        let cols = vec!["Id", "FillingId", "FillingId"];
        for (rel, col) in relations.into_iter().zip(cols) {
            assert_eq!(rel.get_column_camel_case(), col);
        }
    }

    #[test]
    fn test_get_ref_column_camel_case() {
        let relations = setup();
        let ref_cols = vec!["CakeId", "Id", "Id"];
        for (rel, ref_col) in relations.into_iter().zip(ref_cols) {
            assert_eq!(rel.get_ref_column_camel_case(), ref_col);
        }
    }
}
