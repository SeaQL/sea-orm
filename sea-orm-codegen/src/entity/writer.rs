use crate::{ActiveEnum, Entity};
use heck::CamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::{collections::HashMap, str::FromStr};
use strum::EnumIter;
use syn::{punctuated::Punctuated, token::Comma};

#[derive(Clone, Debug)]
pub struct EntityWriter {
    pub(crate) entities: Vec<Entity>,
    pub(crate) enums: HashMap<String, ActiveEnum>,
}

pub struct WriterOutput {
    pub files: Vec<OutputFile>,
}

pub struct OutputFile {
    pub name: String,
    pub content: String,
}

#[derive(PartialEq, Debug)]
pub enum WithSerde {
    None,
    Serialize,
    Deserialize,
    Both,
}

#[derive(Clone, Copy, PartialEq, Debug, EnumIter)]
pub enum DbBackend {
    MySql,
    Sqlite,
    Postgres,
}

impl WithSerde {
    pub fn extra_derive(&self) -> TokenStream {
        let mut extra_derive = match self {
            Self::None => {
                quote! {}
            }
            Self::Serialize => {
                quote! {
                    Serialize
                }
            }
            Self::Deserialize => {
                quote! {
                    Deserialize
                }
            }
            Self::Both => {
                quote! {
                    Serialize, Deserialize
                }
            }
        };

        if !extra_derive.is_empty() {
            extra_derive = quote! { , #extra_derive }
        }

        extra_derive
    }
}

impl FromStr for WithSerde {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "none" => Self::None,
            "serialize" => Self::Serialize,
            "deserialize" => Self::Deserialize,
            "both" => Self::Both,
            v => {
                return Err(crate::Error::TransformError(format!(
                    "Unsupported enum variant '{}'",
                    v
                )))
            }
        })
    }
}

impl EntityWriter {
    pub fn generate(
        self,
        expanded_format: bool,
        with_serde: WithSerde,
        db_backend: DbBackend,
    ) -> WriterOutput {
        let mut files = Vec::new();
        files.extend(self.write_entities(expanded_format, &with_serde, &db_backend));
        files.push(self.write_mod());
        files.push(self.write_prelude());
        if !self.enums.is_empty() {
            files.push(self.write_sea_orm_active_enums(&with_serde));
        }
        WriterOutput { files }
    }

    pub fn write_entities(
        &self,
        expanded_format: bool,
        with_serde: &WithSerde,
        db_backend: &DbBackend,
    ) -> Vec<OutputFile> {
        self.entities
            .iter()
            .map(|entity| {
                let mut lines = Vec::new();
                Self::write_doc_comment(&mut lines);
                let code_blocks = if expanded_format {
                    Self::gen_expanded_code_blocks(entity, with_serde, db_backend)
                } else {
                    Self::gen_compact_code_blocks(entity, with_serde, db_backend)
                };
                Self::write(&mut lines, code_blocks);
                OutputFile {
                    name: format!("{}.rs", entity.get_table_name_snake_case()),
                    content: lines.join("\n\n"),
                }
            })
            .collect()
    }

    pub fn write_mod(&self) -> OutputFile {
        let mut lines = Vec::new();
        Self::write_doc_comment(&mut lines);
        let code_blocks: Vec<TokenStream> = self.entities.iter().map(Self::gen_mod).collect();
        Self::write(
            &mut lines,
            vec![quote! {
                pub mod prelude;
            }],
        );
        lines.push("".to_owned());
        Self::write(&mut lines, code_blocks);
        if !self.enums.is_empty() {
            Self::write(
                &mut lines,
                vec![quote! {
                    pub mod sea_orm_active_enums;
                }],
            );
        }
        OutputFile {
            name: "mod.rs".to_owned(),
            content: lines.join("\n"),
        }
    }

    pub fn write_prelude(&self) -> OutputFile {
        let mut lines = Vec::new();
        Self::write_doc_comment(&mut lines);
        let code_blocks = self.entities.iter().map(Self::gen_prelude_use).collect();
        Self::write(&mut lines, code_blocks);
        OutputFile {
            name: "prelude.rs".to_owned(),
            content: lines.join("\n"),
        }
    }

    pub fn write_sea_orm_active_enums(&self, with_serde: &WithSerde) -> OutputFile {
        let mut lines = Vec::new();
        Self::write_doc_comment(&mut lines);
        Self::write(&mut lines, vec![Self::gen_import(with_serde)]);
        lines.push("".to_owned());
        let code_blocks = self
            .enums
            .iter()
            .map(|(_, active_enum)| active_enum.impl_active_enum(with_serde))
            .collect();
        Self::write(&mut lines, code_blocks);
        OutputFile {
            name: "sea_orm_active_enums.rs".to_owned(),
            content: lines.join("\n"),
        }
    }

    pub fn write(lines: &mut Vec<String>, code_blocks: Vec<TokenStream>) {
        lines.extend(
            code_blocks
                .into_iter()
                .map(|code_block| code_block.to_string())
                .collect::<Vec<_>>(),
        );
    }

    pub fn write_doc_comment(lines: &mut Vec<String>) {
        let ver = env!("CARGO_PKG_VERSION");
        let comments = vec![format!(
            "//! SeaORM Entity. Generated by sea-orm-codegen {}",
            ver
        )];
        lines.extend(comments);
        lines.push("".to_owned());
    }

    pub fn gen_expanded_code_blocks(
        entity: &Entity,
        with_serde: &WithSerde,
        db_backend: &DbBackend,
    ) -> Vec<TokenStream> {
        let mut imports = Self::gen_import(with_serde);
        imports.extend(Self::gen_import_active_enum(entity));
        let mut code_blocks = vec![
            imports,
            Self::gen_entity_struct(),
            Self::gen_impl_entity_name(entity),
            Self::gen_model_struct(entity, with_serde, db_backend),
            Self::gen_column_enum(entity),
            Self::gen_primary_key_enum(entity),
            Self::gen_impl_primary_key(entity, db_backend),
            Self::gen_relation_enum(entity),
            Self::gen_impl_column_trait(entity),
            Self::gen_impl_relation_trait(entity),
        ];
        code_blocks.extend(Self::gen_impl_related(entity));
        code_blocks.extend(Self::gen_impl_conjunct_related(entity));
        code_blocks.extend(vec![Self::gen_impl_active_model_behavior()]);
        code_blocks
    }

    pub fn gen_compact_code_blocks(
        entity: &Entity,
        with_serde: &WithSerde,
        db_backend: &DbBackend,
    ) -> Vec<TokenStream> {
        let mut imports = Self::gen_import(with_serde);
        imports.extend(Self::gen_import_active_enum(entity));
        let mut code_blocks = vec![
            imports,
            Self::gen_compact_model_struct(entity, with_serde, db_backend),
        ];
        let relation_defs = if entity.get_relation_enum_name().is_empty() {
            vec![
                Self::gen_relation_enum(entity),
                Self::gen_impl_relation_trait(entity),
            ]
        } else {
            vec![Self::gen_compact_relation_enum(entity)]
        };
        code_blocks.extend(relation_defs);
        code_blocks.extend(Self::gen_impl_related(entity));
        code_blocks.extend(Self::gen_impl_conjunct_related(entity));
        code_blocks.extend(vec![Self::gen_impl_active_model_behavior()]);
        code_blocks
    }

    pub fn gen_import(with_serde: &WithSerde) -> TokenStream {
        let prelude_import = quote!(
            use sea_orm::entity::prelude::*;
        );

        match with_serde {
            WithSerde::None => prelude_import,
            WithSerde::Serialize => {
                quote! {
                    #prelude_import
                    use serde::Serialize;
                }
            }

            WithSerde::Deserialize => {
                quote! {
                    #prelude_import
                    use serde::Deserialize;
                }
            }

            WithSerde::Both => {
                quote! {
                    #prelude_import
                    use serde::{Deserialize,Serialize};
                }
            }
        }
    }

    pub fn gen_entity_struct() -> TokenStream {
        quote! {
            #[derive(Copy, Clone, Default, Debug, DeriveEntity)]
            pub struct Entity;
        }
    }

    pub fn gen_impl_entity_name(entity: &Entity) -> TokenStream {
        let table_name = entity.table_name.as_str();
        quote! {
            impl EntityName for Entity {
                fn table_name(&self) -> &str {
                    #table_name
                }
            }
        }
    }

    pub fn gen_import_active_enum(entity: &Entity) -> TokenStream {
        entity
            .columns
            .iter()
            .fold(TokenStream::new(), |mut ts, col| {
                if let sea_query::ColumnType::Enum(enum_name, _) = &col.col_type {
                    let enum_name = format_ident!("{}", enum_name.to_camel_case());
                    ts.extend(vec![quote! {
                        use super::sea_orm_active_enums::#enum_name;
                    }]);
                }
                ts
            })
    }

    pub fn gen_model_struct(
        entity: &Entity,
        with_serde: &WithSerde,
        db_backend: &DbBackend,
    ) -> TokenStream {
        let column_names_snake_case = entity.get_column_names_snake_case();
        let column_rs_types = entity.get_column_rs_types(db_backend);

        let extra_derive = with_serde.extra_derive();

        quote! {
            #[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel #extra_derive)]
            pub struct Model {
                #(pub #column_names_snake_case: #column_rs_types,)*
            }
        }
    }

    pub fn gen_column_enum(entity: &Entity) -> TokenStream {
        let column_variants = entity.columns.iter().map(|col| {
            let variant = col.get_name_camel_case();
            let mut variant = quote! { #variant };
            if !col.is_snake_case_name() {
                let column_name = &col.name;
                variant = quote! {
                    #[sea_orm(column_name = #column_name)]
                    #variant
                };
            }
            variant
        });
        quote! {
            #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
            pub enum Column {
                #(#column_variants,)*
            }
        }
    }

    pub fn gen_primary_key_enum(entity: &Entity) -> TokenStream {
        let primary_key_names_camel_case = entity.get_primary_key_names_camel_case();
        quote! {
            #[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
            pub enum PrimaryKey {
                #(#primary_key_names_camel_case,)*
            }
        }
    }

    pub fn gen_impl_primary_key(entity: &Entity, db_backend: &DbBackend) -> TokenStream {
        let primary_key_auto_increment = entity.get_primary_key_auto_increment();
        let value_type = entity.get_primary_key_rs_type(db_backend);
        quote! {
            impl PrimaryKeyTrait for PrimaryKey {
                type ValueType = #value_type;

                fn auto_increment() -> bool {
                    #primary_key_auto_increment
                }
            }
        }
    }

    pub fn gen_relation_enum(entity: &Entity) -> TokenStream {
        let relation_enum_name = entity.get_relation_enum_name();
        quote! {
            #[derive(Copy, Clone, Debug, EnumIter)]
            pub enum Relation {
                #(#relation_enum_name,)*
            }
        }
    }

    pub fn gen_impl_column_trait(entity: &Entity) -> TokenStream {
        let column_names_camel_case = entity.get_column_names_camel_case();
        let column_defs = entity.get_column_defs();
        quote! {
            impl ColumnTrait for Column {
                type EntityName = Entity;

                fn def(&self) -> ColumnDef {
                    match self {
                        #(Self::#column_names_camel_case => #column_defs,)*
                    }
                }
            }
        }
    }

    pub fn gen_impl_relation_trait(entity: &Entity) -> TokenStream {
        let relation_enum_name = entity.get_relation_enum_name();
        let relation_defs = entity.get_relation_defs();
        let quoted = if relation_enum_name.is_empty() {
            quote! {
                panic!("No RelationDef")
            }
        } else {
            quote! {
                match self {
                    #(Self::#relation_enum_name => #relation_defs,)*
                }
            }
        };
        quote! {
            impl RelationTrait for Relation {
                fn def(&self) -> RelationDef {
                    #quoted
                }
            }
        }
    }

    pub fn gen_impl_related(entity: &Entity) -> Vec<TokenStream> {
        entity
            .relations
            .iter()
            .filter(|rel| !rel.self_referencing && rel.num_suffix == 0)
            .map(|rel| {
                let enum_name = rel.get_enum_name();
                let module_name = rel.get_module_name();
                let inner = quote! {
                    fn to() -> RelationDef {
                        Relation::#enum_name.def()
                    }
                };
                if module_name.is_some() {
                    quote! {
                        impl Related<super::#module_name::Entity> for Entity { #inner }
                    }
                } else {
                    quote! {
                        impl Related<Entity> for Entity { #inner }
                    }
                }
            })
            .collect()
    }

    pub fn gen_impl_conjunct_related(entity: &Entity) -> Vec<TokenStream> {
        let table_name_camel_case = entity.get_table_name_camel_case_ident();
        let via_snake_case = entity.get_conjunct_relations_via_snake_case();
        let to_snake_case = entity.get_conjunct_relations_to_snake_case();
        let to_camel_case = entity.get_conjunct_relations_to_camel_case();
        via_snake_case
            .into_iter()
            .zip(to_snake_case)
            .zip(to_camel_case)
            .map(|((via_snake_case, to_snake_case), to_camel_case)| {
                quote! {
                    impl Related<super::#to_snake_case::Entity> for Entity {
                        fn to() -> RelationDef {
                            super::#via_snake_case::Relation::#to_camel_case.def()
                        }

                        fn via() -> Option<RelationDef> {
                            Some(super::#via_snake_case::Relation::#table_name_camel_case.def().rev())
                        }
                    }
                }
            })
            .collect()
    }

    pub fn gen_impl_active_model_behavior() -> TokenStream {
        quote! {
            impl ActiveModelBehavior for ActiveModel {}
        }
    }

    pub fn gen_mod(entity: &Entity) -> TokenStream {
        let table_name_snake_case_ident = entity.get_table_name_snake_case_ident();
        quote! {
            pub mod #table_name_snake_case_ident;
        }
    }

    pub fn gen_prelude_use(entity: &Entity) -> TokenStream {
        let table_name_snake_case_ident = entity.get_table_name_snake_case_ident();
        let table_name_camel_case_ident = entity.get_table_name_camel_case_ident();
        quote! {
            pub use super::#table_name_snake_case_ident::Entity as #table_name_camel_case_ident;
        }
    }

    pub fn gen_compact_model_struct(
        entity: &Entity,
        with_serde: &WithSerde,
        db_backend: &DbBackend,
    ) -> TokenStream {
        let table_name = entity.table_name.as_str();
        let column_names_snake_case = entity.get_column_names_snake_case();
        let column_rs_types = entity.get_column_rs_types(db_backend);
        let primary_keys: Vec<String> = entity
            .primary_keys
            .iter()
            .map(|pk| pk.name.clone())
            .collect();
        let attrs: Vec<TokenStream> = entity
            .columns
            .iter()
            .map(|col| {
                let mut attrs: Punctuated<_, Comma> = Punctuated::new();
                if !col.is_snake_case_name() {
                    let column_name = &col.name;
                    attrs.push(quote! { column_name = #column_name });
                }
                if primary_keys.contains(&col.name) {
                    attrs.push(quote! { primary_key });
                    if !col.auto_increment {
                        attrs.push(quote! { auto_increment = false });
                    }
                }
                if let Some(ts) = col.get_col_type_attrs() {
                    attrs.extend(vec![ts]);
                    if !col.not_null {
                        attrs.push(quote! { nullable });
                    }
                };
                if col.unique {
                    attrs.push(quote! { unique });
                }
                if !attrs.is_empty() {
                    let mut ts = TokenStream::new();
                    for (i, attr) in attrs.into_iter().enumerate() {
                        if i > 0 {
                            ts = quote! { #ts, };
                        }
                        ts = quote! { #ts #attr };
                    }
                    quote! {
                        #[sea_orm(#ts)]
                    }
                } else {
                    TokenStream::new()
                }
            })
            .collect();

        let extra_derive = with_serde.extra_derive();

        quote! {
            #[derive(Clone, Debug, PartialEq, DeriveEntityModel #extra_derive)]
            #[sea_orm(table_name = #table_name)]
            pub struct Model {
                #(
                    #attrs
                    pub #column_names_snake_case: #column_rs_types,
                )*
            }
        }
    }

    pub fn gen_compact_relation_enum(entity: &Entity) -> TokenStream {
        let relation_enum_name = entity.get_relation_enum_name();
        let attrs = entity.get_relation_attrs();
        quote! {
            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            pub enum Relation {
                #(
                    #attrs
                    #relation_enum_name,
                )*
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Column, ConjunctRelation, DbBackend, Entity, EntityWriter, PrimaryKey, Relation,
        RelationType, WithSerde,
    };
    use pretty_assertions::assert_eq;
    use proc_macro2::TokenStream;
    use sea_query::{ColumnType, ForeignKeyAction};
    use std::io::{self, BufRead, BufReader};
    use strum::IntoEnumIterator;

    fn setup() -> Vec<Entity> {
        vec![
            Entity {
                table_name: "cake".to_owned(),
                columns: vec![
                    Column {
                        name: "id".to_owned(),
                        col_type: ColumnType::Integer(Some(11)),
                        auto_increment: true,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "name".to_owned(),
                        col_type: ColumnType::Text,
                        auto_increment: false,
                        not_null: false,
                        unique: false,
                    },
                ],
                relations: vec![Relation {
                    ref_table: "fruit".to_owned(),
                    columns: vec![],
                    ref_columns: vec![],
                    rel_type: RelationType::HasMany,
                    on_delete: None,
                    on_update: None,
                    self_referencing: false,
                    num_suffix: 0,
                }],
                conjunct_relations: vec![ConjunctRelation {
                    via: "cake_filling".to_owned(),
                    to: "filling".to_owned(),
                }],
                primary_keys: vec![PrimaryKey {
                    name: "id".to_owned(),
                }],
            },
            Entity {
                table_name: "_cake_filling_".to_owned(),
                columns: vec![
                    Column {
                        name: "cake_id".to_owned(),
                        col_type: ColumnType::Integer(Some(11)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "filling_id".to_owned(),
                        col_type: ColumnType::Integer(Some(11)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                ],
                relations: vec![
                    Relation {
                        ref_table: "cake".to_owned(),
                        columns: vec!["cake_id".to_owned()],
                        ref_columns: vec!["id".to_owned()],
                        rel_type: RelationType::BelongsTo,
                        on_delete: Some(ForeignKeyAction::Cascade),
                        on_update: Some(ForeignKeyAction::Cascade),
                        self_referencing: false,
                        num_suffix: 0,
                    },
                    Relation {
                        ref_table: "filling".to_owned(),
                        columns: vec!["filling_id".to_owned()],
                        ref_columns: vec!["id".to_owned()],
                        rel_type: RelationType::BelongsTo,
                        on_delete: Some(ForeignKeyAction::Cascade),
                        on_update: Some(ForeignKeyAction::Cascade),
                        self_referencing: false,
                        num_suffix: 0,
                    },
                ],
                conjunct_relations: vec![],
                primary_keys: vec![
                    PrimaryKey {
                        name: "cake_id".to_owned(),
                    },
                    PrimaryKey {
                        name: "filling_id".to_owned(),
                    },
                ],
            },
            Entity {
                table_name: "filling".to_owned(),
                columns: vec![
                    Column {
                        name: "id".to_owned(),
                        col_type: ColumnType::Integer(Some(11)),
                        auto_increment: true,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "name".to_owned(),
                        col_type: ColumnType::String(Some(255)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                ],
                relations: vec![],
                conjunct_relations: vec![ConjunctRelation {
                    via: "cake_filling".to_owned(),
                    to: "cake".to_owned(),
                }],
                primary_keys: vec![PrimaryKey {
                    name: "id".to_owned(),
                }],
            },
            Entity {
                table_name: "fruit".to_owned(),
                columns: vec![
                    Column {
                        name: "id".to_owned(),
                        col_type: ColumnType::Integer(Some(11)),
                        auto_increment: true,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "name".to_owned(),
                        col_type: ColumnType::String(Some(255)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "cake_id".to_owned(),
                        col_type: ColumnType::Integer(Some(11)),
                        auto_increment: false,
                        not_null: false,
                        unique: false,
                    },
                ],
                relations: vec![
                    Relation {
                        ref_table: "cake".to_owned(),
                        columns: vec!["cake_id".to_owned()],
                        ref_columns: vec!["id".to_owned()],
                        rel_type: RelationType::BelongsTo,
                        on_delete: None,
                        on_update: None,
                        self_referencing: false,
                        num_suffix: 0,
                    },
                    Relation {
                        ref_table: "vendor".to_owned(),
                        columns: vec![],
                        ref_columns: vec![],
                        rel_type: RelationType::HasMany,
                        on_delete: None,
                        on_update: None,
                        self_referencing: false,
                        num_suffix: 0,
                    },
                ],
                conjunct_relations: vec![],
                primary_keys: vec![PrimaryKey {
                    name: "id".to_owned(),
                }],
            },
            Entity {
                table_name: "vendor".to_owned(),
                columns: vec![
                    Column {
                        name: "id".to_owned(),
                        col_type: ColumnType::Integer(Some(11)),
                        auto_increment: true,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "_name_".to_owned(),
                        col_type: ColumnType::String(Some(255)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "fruitId".to_owned(),
                        col_type: ColumnType::Integer(Some(11)),
                        auto_increment: false,
                        not_null: false,
                        unique: false,
                    },
                ],
                relations: vec![Relation {
                    ref_table: "fruit".to_owned(),
                    columns: vec!["fruitId".to_owned()],
                    ref_columns: vec!["id".to_owned()],
                    rel_type: RelationType::BelongsTo,
                    on_delete: None,
                    on_update: None,
                    self_referencing: false,
                    num_suffix: 0,
                }],
                conjunct_relations: vec![],
                primary_keys: vec![PrimaryKey {
                    name: "id".to_owned(),
                }],
            },
            Entity {
                table_name: "rust_keyword".to_owned(),
                columns: vec![
                    Column {
                        name: "id".to_owned(),
                        col_type: ColumnType::Integer(Some(11)),
                        auto_increment: true,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "testing".to_owned(),
                        col_type: ColumnType::TinyInteger(Some(11)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "rust".to_owned(),
                        col_type: ColumnType::TinyUnsigned(Some(11)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "keywords".to_owned(),
                        col_type: ColumnType::SmallInteger(Some(11)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "type".to_owned(),
                        col_type: ColumnType::SmallUnsigned(Some(11)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "typeof".to_owned(),
                        col_type: ColumnType::Integer(Some(11)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "crate".to_owned(),
                        col_type: ColumnType::Unsigned(Some(11)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "self".to_owned(),
                        col_type: ColumnType::BigInteger(Some(11)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "self_id1".to_owned(),
                        col_type: ColumnType::BigUnsigned(Some(11)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "self_id2".to_owned(),
                        col_type: ColumnType::Integer(Some(11)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "fruit_id1".to_owned(),
                        col_type: ColumnType::Integer(Some(11)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "fruit_id2".to_owned(),
                        col_type: ColumnType::Integer(Some(11)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "cake_id".to_owned(),
                        col_type: ColumnType::Integer(Some(11)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                ],
                relations: vec![
                    Relation {
                        ref_table: "rust_keyword".to_owned(),
                        columns: vec!["self_id1".to_owned()],
                        ref_columns: vec!["id".to_owned()],
                        rel_type: RelationType::BelongsTo,
                        on_delete: None,
                        on_update: None,
                        self_referencing: true,
                        num_suffix: 1,
                    },
                    Relation {
                        ref_table: "rust_keyword".to_owned(),
                        columns: vec!["self_id2".to_owned()],
                        ref_columns: vec!["id".to_owned()],
                        rel_type: RelationType::BelongsTo,
                        on_delete: None,
                        on_update: None,
                        self_referencing: true,
                        num_suffix: 2,
                    },
                    Relation {
                        ref_table: "fruit".to_owned(),
                        columns: vec!["fruit_id1".to_owned()],
                        ref_columns: vec!["id".to_owned()],
                        rel_type: RelationType::BelongsTo,
                        on_delete: None,
                        on_update: None,
                        self_referencing: false,
                        num_suffix: 1,
                    },
                    Relation {
                        ref_table: "fruit".to_owned(),
                        columns: vec!["fruit_id2".to_owned()],
                        ref_columns: vec!["id".to_owned()],
                        rel_type: RelationType::BelongsTo,
                        on_delete: None,
                        on_update: None,
                        self_referencing: false,
                        num_suffix: 2,
                    },
                    Relation {
                        ref_table: "cake".to_owned(),
                        columns: vec!["cake_id".to_owned()],
                        ref_columns: vec!["id".to_owned()],
                        rel_type: RelationType::BelongsTo,
                        on_delete: None,
                        on_update: None,
                        self_referencing: false,
                        num_suffix: 0,
                    },
                ],
                conjunct_relations: vec![],
                primary_keys: vec![PrimaryKey {
                    name: "id".to_owned(),
                }],
            },
        ]
    }

    #[test]
    fn test_gen_expanded_code_blocks() -> io::Result<()> {
        let entities = setup();
        const ENTITY_FILES: [&str; 6] = [
            include_str!("../../tests/expanded/cake.rs"),
            include_str!("../../tests/expanded/cake_filling.rs"),
            include_str!("../../tests/expanded/filling.rs"),
            include_str!("../../tests/expanded/fruit.rs"),
            include_str!("../../tests/expanded/vendor.rs"),
            include_str!("../../tests/expanded/rust_keyword.rs"),
        ];

        assert_eq!(entities.len(), ENTITY_FILES.len());

        for (i, entity) in entities.iter().enumerate() {
            let mut reader = BufReader::new(ENTITY_FILES[i].as_bytes());
            let mut lines: Vec<String> = Vec::new();

            reader.read_until(b';', &mut Vec::new())?;

            let mut line = String::new();
            while reader.read_line(&mut line)? > 0 {
                lines.push(line.to_owned());
                line.clear();
            }
            let content = lines.join("");
            let expected: TokenStream = content.parse().unwrap();
            for db_backend in DbBackend::iter() {
                let generated = EntityWriter::gen_expanded_code_blocks(
                    entity,
                    &crate::WithSerde::None,
                    &db_backend,
                )
                .into_iter()
                .skip(1)
                .fold(TokenStream::new(), |mut acc, tok| {
                    acc.extend(tok);
                    acc
                });
                assert_eq!(expected.to_string(), generated.to_string());
            }
        }

        Ok(())
    }

    #[test]
    fn test_gen_compact_code_blocks() -> io::Result<()> {
        let entities = setup();
        const ENTITY_FILES: [&str; 6] = [
            include_str!("../../tests/compact/cake.rs"),
            include_str!("../../tests/compact/cake_filling.rs"),
            include_str!("../../tests/compact/filling.rs"),
            include_str!("../../tests/compact/fruit.rs"),
            include_str!("../../tests/compact/vendor.rs"),
            include_str!("../../tests/compact/rust_keyword.rs"),
        ];

        assert_eq!(entities.len(), ENTITY_FILES.len());

        for (i, entity) in entities.iter().enumerate() {
            let mut reader = BufReader::new(ENTITY_FILES[i].as_bytes());
            let mut lines: Vec<String> = Vec::new();

            reader.read_until(b';', &mut Vec::new())?;

            let mut line = String::new();
            while reader.read_line(&mut line)? > 0 {
                lines.push(line.to_owned());
                line.clear();
            }
            let content = lines.join("");
            let expected: TokenStream = content.parse().unwrap();
            for db_backend in DbBackend::iter() {
                let generated = EntityWriter::gen_compact_code_blocks(
                    entity,
                    &crate::WithSerde::None,
                    &db_backend,
                )
                .into_iter()
                .skip(1)
                .fold(TokenStream::new(), |mut acc, tok| {
                    acc.extend(tok);
                    acc
                });
                assert_eq!(expected.to_string(), generated.to_string());
            }
        }

        Ok(())
    }

    #[test]
    fn test_gen_with_serde() -> io::Result<()> {
        let cake_entity = setup().get(0).unwrap().clone();

        assert_eq!(cake_entity.get_table_name_snake_case(), "cake");

        for db_backend in DbBackend::iter() {
            // Compact code blocks
            assert_serde_variant_results(
                &cake_entity,
                &(
                    include_str!("../../tests/compact_with_serde/cake_none.rs").into(),
                    WithSerde::None,
                    db_backend,
                ),
                Box::new(EntityWriter::gen_compact_code_blocks),
            )?;
            assert_serde_variant_results(
                &cake_entity,
                &(
                    include_str!("../../tests/compact_with_serde/cake_serialize.rs").into(),
                    WithSerde::Serialize,
                    db_backend,
                ),
                Box::new(EntityWriter::gen_compact_code_blocks),
            )?;
            assert_serde_variant_results(
                &cake_entity,
                &(
                    include_str!("../../tests/compact_with_serde/cake_deserialize.rs").into(),
                    WithSerde::Deserialize,
                    db_backend,
                ),
                Box::new(EntityWriter::gen_compact_code_blocks),
            )?;
            assert_serde_variant_results(
                &cake_entity,
                &(
                    include_str!("../../tests/compact_with_serde/cake_both.rs").into(),
                    WithSerde::Both,
                    db_backend,
                ),
                Box::new(EntityWriter::gen_compact_code_blocks),
            )?;

            // Expanded code blocks
            assert_serde_variant_results(
                &cake_entity,
                &(
                    include_str!("../../tests/expanded_with_serde/cake_none.rs").into(),
                    WithSerde::None,
                    db_backend,
                ),
                Box::new(EntityWriter::gen_expanded_code_blocks),
            )?;
            assert_serde_variant_results(
                &cake_entity,
                &(
                    include_str!("../../tests/expanded_with_serde/cake_serialize.rs").into(),
                    WithSerde::Serialize,
                    db_backend,
                ),
                Box::new(EntityWriter::gen_expanded_code_blocks),
            )?;
            assert_serde_variant_results(
                &cake_entity,
                &(
                    include_str!("../../tests/expanded_with_serde/cake_deserialize.rs").into(),
                    WithSerde::Deserialize,
                    db_backend,
                ),
                Box::new(EntityWriter::gen_expanded_code_blocks),
            )?;
            assert_serde_variant_results(
                &cake_entity,
                &(
                    include_str!("../../tests/expanded_with_serde/cake_both.rs").into(),
                    WithSerde::Both,
                    db_backend,
                ),
                Box::new(EntityWriter::gen_expanded_code_blocks),
            )?;
        }

        Ok(())
    }

    fn assert_serde_variant_results(
        cake_entity: &Entity,
        entity_serde_variant: &(String, WithSerde, DbBackend),
        generator: Box<dyn Fn(&Entity, &WithSerde, &DbBackend) -> Vec<TokenStream>>,
    ) -> io::Result<()> {
        let mut reader = BufReader::new(entity_serde_variant.0.as_bytes());
        let mut lines: Vec<String> = Vec::new();

        reader.read_until(b'\n', &mut Vec::new())?;

        let mut line = String::new();
        while reader.read_line(&mut line)? > 0 {
            lines.push(line.to_owned());
            line.clear();
        }
        let content = lines.join("");
        let expected: TokenStream = content.parse().unwrap();
        let generated = generator(
            cake_entity,
            &entity_serde_variant.1,
            &entity_serde_variant.2,
        )
        .into_iter()
        .fold(TokenStream::new(), |mut acc, tok| {
            acc.extend(tok);
            acc
        });

        assert_eq!(expected.to_string(), generated.to_string());
        Ok(())
    }
}
