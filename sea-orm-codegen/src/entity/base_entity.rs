use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro2::{Ident, TokenStream};
use quote::format_ident;
use quote::quote;
use sea_query::ColumnType;

use crate::{
    util::escape_rust_keyword, Column, ConjunctRelation, DateTimeCrate, PrimaryKey, Relation,
};

#[derive(Clone, Debug)]
pub struct Entity {
    pub(crate) table_name: String,
    pub(crate) columns: Vec<Column>,
    pub(crate) relations: Vec<Relation>,
    pub(crate) conjunct_relations: Vec<ConjunctRelation>,
    pub(crate) primary_keys: Vec<PrimaryKey>,
    pub(crate) comment: Option<String>,
}

impl Entity {
    pub fn get_table_comment(&self) -> Option<&String> {
        self.comment.as_ref()
    }

    pub fn get_table_name_snake_case(&self) -> String {
        self.table_name.to_snake_case()
    }

    pub fn get_table_name_camel_case(&self) -> String {
        self.table_name.to_upper_camel_case()
    }

    pub fn get_table_name_snake_case_ident(&self) -> Ident {
        format_ident!("{}", escape_rust_keyword(self.get_table_name_snake_case()))
    }

    pub fn get_table_name_camel_case_ident(&self) -> Ident {
        format_ident!("{}", escape_rust_keyword(self.get_table_name_camel_case()))
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

    pub fn get_column_rs_types(&self, date_time_crate: &DateTimeCrate) -> Vec<TokenStream> {
        self.columns
            .clone()
            .into_iter()
            .map(|col| col.get_rs_type(date_time_crate))
            .collect()
    }

    pub fn get_column_defs(&self) -> Vec<TokenStream> {
        self.columns
            .clone()
            .into_iter()
            .map(|col| col.get_def())
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

    pub fn get_relation_module_name(&self) -> Vec<Option<Ident>> {
        self.relations
            .iter()
            .map(|rel| rel.get_module_name())
            .collect()
    }

    pub fn get_relation_enum_name(&self) -> Vec<Ident> {
        self.relations
            .iter()
            .map(|rel| rel.get_enum_name())
            .collect()
    }

    /// Used to generate the names for the `enum RelatedEntity` that is useful to the Seaography project
    pub fn get_related_entity_enum_name(&self) -> Vec<Ident> {
        // 1st step get conjunct relations data
        let conjunct_related_names = self.get_conjunct_relations_to_upper_camel_case();

        // 2nd step get reverse self relations data
        let self_relations_reverse = self
            .relations
            .iter()
            .filter(|rel| rel.self_referencing)
            .map(|rel| format_ident!("{}Reverse", rel.get_enum_name()));

        // 3rd step get normal relations data
        self.get_relation_enum_name()
            .into_iter()
            .chain(self_relations_reverse)
            .chain(conjunct_related_names)
            .collect()
    }

    pub fn get_relation_defs(&self) -> Vec<TokenStream> {
        self.relations.iter().map(|rel| rel.get_def()).collect()
    }

    pub fn get_relation_attrs(&self) -> Vec<TokenStream> {
        self.relations.iter().map(|rel| rel.get_attrs()).collect()
    }

    /// Used to generate the attributes for the `enum RelatedEntity` that is useful to the Seaography project
    pub fn get_related_entity_attrs(&self) -> Vec<TokenStream> {
        // 1st step get conjunct relations data
        let conjunct_related_attrs = self.conjunct_relations.iter().map(|conj| {
            let entity = format!("super::{}::Entity", conj.get_to_snake_case());

            quote! {
                #[sea_orm(
                    entity = #entity
                )]
            }
        });

        // helper function that generates attributes for `Relation` data
        let produce_relation_attrs = |rel: &Relation, reverse: bool| {
            let entity = match rel.get_module_name() {
                Some(module_name) => format!("super::{}::Entity", module_name),
                None => String::from("Entity"),
            };

            if rel.self_referencing || !rel.impl_related || rel.num_suffix > 0 {
                let def = if reverse {
                    format!("Relation::{}.def().rev()", rel.get_enum_name())
                } else {
                    format!("Relation::{}.def()", rel.get_enum_name())
                };

                quote! {
                    #[sea_orm(
                        entity = #entity,
                        def = #def
                    )]
                }
            } else {
                quote! {
                    #[sea_orm(
                        entity = #entity
                    )]
                }
            }
        };

        // 2nd step get reverse self relations data
        let self_relations_reverse_attrs = self
            .relations
            .iter()
            .filter(|rel| rel.self_referencing)
            .map(|rel| produce_relation_attrs(rel, true));

        // 3rd step get normal relations data
        self.relations
            .iter()
            .map(|rel| produce_relation_attrs(rel, false))
            .chain(self_relations_reverse_attrs)
            .chain(conjunct_related_attrs)
            .collect()
    }

    pub fn get_primary_key_auto_increment(&self) -> Ident {
        let auto_increment = self.columns.iter().any(|col| col.auto_increment);
        format_ident!("{}", auto_increment)
    }

    pub fn get_primary_key_rs_type(&self, date_time_crate: &DateTimeCrate) -> TokenStream {
        let types = self
            .primary_keys
            .iter()
            .map(|primary_key| {
                self.columns
                    .iter()
                    .find(|col| col.name.eq(&primary_key.name))
                    .unwrap()
                    .get_rs_type(date_time_crate)
                    .to_string()
            })
            .collect::<Vec<_>>();
        if !types.is_empty() {
            let value_type = if types.len() > 1 {
                vec!["(".to_owned(), types.join(", "), ")".to_owned()]
            } else {
                types
            };
            value_type.join("").parse().unwrap()
        } else {
            TokenStream::new()
        }
    }

    pub fn get_conjunct_relations_via_snake_case(&self) -> Vec<Ident> {
        self.conjunct_relations
            .iter()
            .map(|con_rel| con_rel.get_via_snake_case())
            .collect()
    }

    pub fn get_conjunct_relations_to_snake_case(&self) -> Vec<Ident> {
        self.conjunct_relations
            .iter()
            .map(|con_rel| con_rel.get_to_snake_case())
            .collect()
    }

    pub fn get_conjunct_relations_to_upper_camel_case(&self) -> Vec<Ident> {
        self.conjunct_relations
            .iter()
            .map(|con_rel| con_rel.get_to_upper_camel_case())
            .collect()
    }

    pub fn get_eq_needed(&self) -> TokenStream {
        fn is_floats(col_type: &ColumnType) -> bool {
            match col_type {
                ColumnType::Float | ColumnType::Double => true,
                ColumnType::Array(col_type) => is_floats(col_type),
                _ => false,
            }
        }
        self.columns
            .iter()
            .find(|column| is_floats(&column.col_type))
            // check if float or double exist.
            // if exist, return nothing
            .map_or(quote! {, Eq}, |_| quote! {})
    }

    pub fn get_column_serde_attributes(
        &self,
        serde_skip_deserializing_primary_key: bool,
        serde_skip_hidden_column: bool,
    ) -> Vec<TokenStream> {
        self.columns
            .iter()
            .map(|col| {
                let is_primary_key = self.primary_keys.iter().any(|pk| pk.name == col.name);
                col.get_serde_attribute(
                    is_primary_key,
                    serde_skip_deserializing_primary_key,
                    serde_skip_hidden_column,
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use quote::{format_ident, quote};
    use sea_query::{ColumnType, ForeignKeyAction};

    use crate::{Column, DateTimeCrate, Entity, PrimaryKey, Relation, RelationType};

    fn setup() -> Entity {
        Entity {
            table_name: "special_cake".to_owned(),
            columns: vec![
                Column {
                    name: "id".to_owned(),
                    col_type: ColumnType::Integer,
                    auto_increment: false,
                    not_null: false,
                    unique: false,
                },
                Column {
                    name: "name".to_owned(),
                    col_type: ColumnType::String(None),
                    auto_increment: false,
                    not_null: false,
                    unique: false,
                },
            ],
            relations: vec![
                Relation {
                    ref_table: "fruit".to_owned(),
                    columns: vec!["id".to_owned()],
                    ref_columns: vec!["cake_id".to_owned()],
                    rel_type: RelationType::HasOne,
                    on_delete: Some(ForeignKeyAction::Cascade),
                    on_update: Some(ForeignKeyAction::Cascade),
                    self_referencing: false,
                    num_suffix: 0,
                    impl_related: true,
                },
                Relation {
                    ref_table: "filling".to_owned(),
                    columns: vec!["id".to_owned()],
                    ref_columns: vec!["cake_id".to_owned()],
                    rel_type: RelationType::HasOne,
                    on_delete: Some(ForeignKeyAction::Cascade),
                    on_update: Some(ForeignKeyAction::Cascade),
                    self_referencing: false,
                    num_suffix: 0,
                    impl_related: true,
                },
            ],
            conjunct_relations: vec![],
            primary_keys: vec![PrimaryKey {
                name: "id".to_owned(),
            }],
            comment: None,
        }
    }

    #[test]
    fn test_get_table_name_snake_case() {
        let entity = setup();

        assert_eq!(
            entity.get_table_name_snake_case(),
            "special_cake".to_owned()
        );
    }

    #[test]
    fn test_get_table_name_camel_case() {
        let entity = setup();

        assert_eq!(entity.get_table_name_camel_case(), "SpecialCake".to_owned());
    }

    #[test]
    fn test_get_table_name_snake_case_ident() {
        let entity = setup();

        assert_eq!(
            entity.get_table_name_snake_case_ident(),
            format_ident!("{}", "special_cake")
        );
    }

    #[test]
    fn test_get_table_name_camel_case_ident() {
        let entity = setup();

        assert_eq!(
            entity.get_table_name_camel_case_ident(),
            format_ident!("{}", "SpecialCake")
        );
    }

    #[test]
    fn test_get_column_names_snake_case() {
        let entity = setup();

        for (i, elem) in entity.get_column_names_snake_case().into_iter().enumerate() {
            assert_eq!(elem, entity.columns[i].get_name_snake_case());
        }
    }

    #[test]
    fn test_get_column_names_camel_case() {
        let entity = setup();

        for (i, elem) in entity.get_column_names_camel_case().into_iter().enumerate() {
            assert_eq!(elem, entity.columns[i].get_name_camel_case());
        }
    }

    #[test]
    fn test_get_column_rs_types() {
        let entity = setup();

        for (i, elem) in entity
            .get_column_rs_types(&DateTimeCrate::Chrono)
            .into_iter()
            .enumerate()
        {
            assert_eq!(
                elem.to_string(),
                entity.columns[i]
                    .get_rs_type(&DateTimeCrate::Chrono)
                    .to_string()
            );
        }
    }

    #[test]
    fn test_get_column_defs() {
        let entity = setup();

        for (i, elem) in entity.get_column_defs().into_iter().enumerate() {
            assert_eq!(elem.to_string(), entity.columns[i].get_def().to_string());
        }
    }

    #[test]
    fn test_get_primary_key_names_snake_case() {
        let entity = setup();

        for (i, elem) in entity
            .get_primary_key_names_snake_case()
            .into_iter()
            .enumerate()
        {
            assert_eq!(elem, entity.primary_keys[i].get_name_snake_case());
        }
    }

    #[test]
    fn test_get_primary_key_names_camel_case() {
        let entity = setup();

        for (i, elem) in entity
            .get_primary_key_names_camel_case()
            .into_iter()
            .enumerate()
        {
            assert_eq!(elem, entity.primary_keys[i].get_name_camel_case());
        }
    }

    #[test]
    fn test_get_relation_module_name() {
        let entity = setup();

        for (i, elem) in entity.get_relation_module_name().into_iter().enumerate() {
            assert_eq!(elem, entity.relations[i].get_module_name());
        }
    }

    #[test]
    fn test_get_relation_enum_name() {
        let entity = setup();

        for (i, elem) in entity.get_relation_enum_name().into_iter().enumerate() {
            assert_eq!(elem, entity.relations[i].get_enum_name());
        }
    }

    #[test]
    fn test_get_relation_defs() {
        let entity = setup();

        for (i, elem) in entity.get_relation_defs().into_iter().enumerate() {
            assert_eq!(elem.to_string(), entity.relations[i].get_def().to_string());
        }
    }

    #[test]
    fn test_get_relation_attrs() {
        let entity = setup();

        for (i, elem) in entity.get_relation_attrs().into_iter().enumerate() {
            assert_eq!(
                elem.to_string(),
                entity.relations[i].get_attrs().to_string()
            );
        }
    }

    #[test]
    fn test_get_primary_key_auto_increment() {
        let mut entity = setup();

        assert_eq!(
            entity.get_primary_key_auto_increment(),
            format_ident!("{}", false)
        );

        entity.columns[0].auto_increment = true;
        assert_eq!(
            entity.get_primary_key_auto_increment(),
            format_ident!("{}", true)
        );
    }

    #[test]
    fn test_get_primary_key_rs_type() {
        let entity = setup();

        assert_eq!(
            entity
                .get_primary_key_rs_type(&DateTimeCrate::Chrono)
                .to_string(),
            entity.columns[0]
                .get_rs_type(&DateTimeCrate::Chrono)
                .to_string()
        );
    }

    #[test]
    fn test_get_conjunct_relations_via_snake_case() {
        let entity = setup();

        for (i, elem) in entity
            .get_conjunct_relations_via_snake_case()
            .into_iter()
            .enumerate()
        {
            assert_eq!(elem, entity.conjunct_relations[i].get_via_snake_case());
        }
    }

    #[test]
    fn test_get_conjunct_relations_to_snake_case() {
        let entity = setup();

        for (i, elem) in entity
            .get_conjunct_relations_to_snake_case()
            .into_iter()
            .enumerate()
        {
            assert_eq!(elem, entity.conjunct_relations[i].get_to_snake_case());
        }
    }

    #[test]
    fn test_get_conjunct_relations_to_upper_camel_case() {
        let entity = setup();

        for (i, elem) in entity
            .get_conjunct_relations_to_upper_camel_case()
            .into_iter()
            .enumerate()
        {
            assert_eq!(elem, entity.conjunct_relations[i].get_to_upper_camel_case());
        }
    }

    #[test]
    fn test_get_eq_needed() {
        let entity = setup();
        let expected = quote! {, Eq};

        assert_eq!(entity.get_eq_needed().to_string(), expected.to_string());
    }
}
