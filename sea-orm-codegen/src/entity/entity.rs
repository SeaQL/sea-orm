use crate::{Column, PrimaryKey, Relation};
use heck::{CamelCase, SnakeCase};
use proc_macro2::{Ident, TokenStream};
use quote::format_ident;

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

    pub fn get_table_name_camel_case(&self) -> String {
        self.table_name.to_camel_case()
    }

    pub fn get_table_name_snake_case_ident(&self) -> Ident {
        format_ident!("{}", self.get_table_name_snake_case())
    }

    pub fn get_table_name_camel_case_ident(&self) -> Ident {
        format_ident!("{}", self.get_table_name_camel_case())
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

    pub fn get_column_rs_types(&self) -> Vec<TokenStream> {
        self.columns
            .clone()
            .into_iter()
            .map(|col| col.get_rs_type())
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

    pub fn get_relation_defs(&self) -> Vec<TokenStream> {
        self.relations.iter().map(|rel| rel.get_def()).collect()
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

    pub fn get_primary_key_auto_increment(&self) -> Ident {
        let auto_increment = self.columns.iter().any(|col| col.auto_increment);
        format_ident!("{}", auto_increment)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Column, Entity, PrimaryKey, Relation, RelationType};
    use quote::format_ident;
    use sea_query::ColumnType;

    fn setup() -> Entity {
        Entity {
            table_name: "special_cake".to_owned(),
            columns: vec![
                Column {
                    name: "id".to_owned(),
                    col_type: ColumnType::String(None),
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
                },
                Relation {
                    ref_table: "filling".to_owned(),
                    columns: vec!["id".to_owned()],
                    ref_columns: vec!["cake_id".to_owned()],
                    rel_type: RelationType::HasOne,
                },
            ],
            primary_keys: vec![PrimaryKey {
                name: "id".to_owned(),
            }],
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

        for (i, elem) in entity.get_column_rs_types().into_iter().enumerate() {
            assert_eq!(
                elem.to_string(),
                entity.columns[i].get_rs_type().to_string()
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
    fn test_get_relation_ref_tables_snake_case() {
        let entity = setup();

        for (i, elem) in entity
            .get_relation_ref_tables_snake_case()
            .into_iter()
            .enumerate()
        {
            assert_eq!(elem, entity.relations[i].get_ref_table_snake_case());
        }
    }

    #[test]
    fn test_get_relation_ref_tables_camel_case() {
        let entity = setup();

        for (i, elem) in entity
            .get_relation_ref_tables_camel_case()
            .into_iter()
            .enumerate()
        {
            assert_eq!(elem, entity.relations[i].get_ref_table_camel_case());
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
    fn test_get_relation_rel_types() {
        let entity = setup();

        for (i, elem) in entity.get_relation_rel_types().into_iter().enumerate() {
            assert_eq!(elem, entity.relations[i].get_rel_type());
        }
    }

    #[test]
    fn test_get_relation_columns_camel_case() {
        let entity = setup();

        for (i, elem) in entity
            .get_relation_columns_camel_case()
            .into_iter()
            .enumerate()
        {
            assert_eq!(elem, entity.relations[i].get_column_camel_case());
        }
    }

    #[test]
    fn test_get_relation_ref_columns_camel_case() {
        let entity = setup();

        for (i, elem) in entity
            .get_relation_ref_columns_camel_case()
            .into_iter()
            .enumerate()
        {
            assert_eq!(elem, entity.relations[i].get_ref_column_camel_case());
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
}
