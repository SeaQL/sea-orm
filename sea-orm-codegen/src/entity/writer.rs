use crate::{util::escape_rust_keyword, ActiveEnum, Entity};
use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::{collections::BTreeMap, str::FromStr};
use syn::{punctuated::Punctuated, token::Comma};
use tracing::info;

#[derive(Clone, Debug)]
pub struct EntityWriter {
    pub(crate) entities: Vec<Entity>,
    pub(crate) enums: BTreeMap<String, ActiveEnum>,
}

pub struct WriterOutput {
    pub files: Vec<OutputFile>,
}

pub struct OutputFile {
    pub name: String,
    pub content: String,
}

#[derive(PartialEq, Eq, Debug)]
pub enum WithSerde {
    None,
    Serialize,
    Deserialize,
    Both,
}

#[derive(Debug)]
pub enum DateTimeCrate {
    Chrono,
    Time,
}

#[derive(Debug)]
pub struct EntityWriterContext {
    pub(crate) expanded_format: bool,
    pub(crate) with_serde: WithSerde,
    pub(crate) with_copy_enums: bool,
    pub(crate) date_time_crate: DateTimeCrate,
    pub(crate) schema_name: Option<String>,
    pub(crate) lib: bool,
    pub(crate) serde_skip_hidden_column: bool,
    pub(crate) serde_skip_deserializing_primary_key: bool,
    pub(crate) model_extra_derives: TokenStream,
    pub(crate) model_extra_attributes: TokenStream,
    pub(crate) seaography: bool,
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

/// Converts model_extra_derives argument to token stream
fn bonus_derive<T, I>(model_extra_derives: I) -> TokenStream
where
    T: Into<String>,
    I: IntoIterator<Item = T>,
{
    model_extra_derives
        .into_iter()
        .map(Into::<String>::into)
        .fold(TokenStream::default(), |acc, derive| {
            let tokens: TokenStream = derive.parse().unwrap();
            quote! { #acc, #tokens }
        })
}

/// convert attributes argument to token stream
fn bonus_attributes<T, I>(attributes: I) -> TokenStream
where
    T: Into<String>,
    I: IntoIterator<Item = T>,
{
    attributes.into_iter().map(Into::<String>::into).fold(
        TokenStream::default(),
        |acc, attribute| {
            let tokens: TokenStream = attribute.parse().unwrap();
            quote! {
                #acc
                #[#tokens]
            }
        },
    )
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
                    "Unsupported enum variant '{v}'"
                )))
            }
        })
    }
}

impl EntityWriterContext {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        expanded_format: bool,
        with_serde: WithSerde,
        with_copy_enums: bool,
        date_time_crate: DateTimeCrate,
        schema_name: Option<String>,
        lib: bool,
        serde_skip_deserializing_primary_key: bool,
        serde_skip_hidden_column: bool,
        model_extra_derives: Vec<String>,
        model_extra_attributes: Vec<String>,
        seaography: bool,
    ) -> Self {
        Self {
            expanded_format,
            with_serde,
            with_copy_enums,
            date_time_crate,
            schema_name,
            lib,
            serde_skip_deserializing_primary_key,
            serde_skip_hidden_column,
            model_extra_derives: bonus_derive(model_extra_derives),
            model_extra_attributes: bonus_attributes(model_extra_attributes),
            seaography,
        }
    }
}

impl EntityWriter {
    pub fn generate(self, context: &EntityWriterContext) -> WriterOutput {
        let mut files = Vec::new();
        files.extend(self.write_entities(context));
        files.push(self.write_index_file(context.lib));
        files.push(self.write_prelude());
        if !self.enums.is_empty() {
            files.push(
                self.write_sea_orm_active_enums(&context.with_serde, context.with_copy_enums),
            );
        }
        WriterOutput { files }
    }

    pub fn write_entities(&self, context: &EntityWriterContext) -> Vec<OutputFile> {
        self.entities
            .iter()
            .map(|entity| {
                let entity_file = format!("{}.rs", entity.get_table_name_snake_case());
                let column_info = entity
                    .columns
                    .iter()
                    .map(|column| column.get_info(&context.date_time_crate))
                    .collect::<Vec<String>>();
                // Serde must be enabled to use this
                let serde_skip_deserializing_primary_key = context
                    .serde_skip_deserializing_primary_key
                    && matches!(context.with_serde, WithSerde::Both | WithSerde::Deserialize);
                let serde_skip_hidden_column = context.serde_skip_hidden_column
                    && matches!(
                        context.with_serde,
                        WithSerde::Both | WithSerde::Serialize | WithSerde::Deserialize
                    );

                info!("Generating {}", entity_file);
                for info in column_info.iter() {
                    info!("    > {}", info);
                }

                let mut lines = Vec::new();
                Self::write_doc_comment(&mut lines);
                let code_blocks = if context.expanded_format {
                    Self::gen_expanded_code_blocks(
                        entity,
                        &context.with_serde,
                        &context.date_time_crate,
                        &context.schema_name,
                        serde_skip_deserializing_primary_key,
                        serde_skip_hidden_column,
                        &context.model_extra_derives,
                        &context.model_extra_attributes,
                        context.seaography,
                    )
                } else {
                    Self::gen_compact_code_blocks(
                        entity,
                        &context.with_serde,
                        &context.date_time_crate,
                        &context.schema_name,
                        serde_skip_deserializing_primary_key,
                        serde_skip_hidden_column,
                        &context.model_extra_derives,
                        &context.model_extra_attributes,
                        context.seaography,
                    )
                };
                Self::write(&mut lines, code_blocks);
                OutputFile {
                    name: entity_file,
                    content: lines.join("\n\n"),
                }
            })
            .collect()
    }

    pub fn write_index_file(&self, lib: bool) -> OutputFile {
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

        let file_name = match lib {
            true => "lib.rs".to_owned(),
            false => "mod.rs".to_owned(),
        };

        OutputFile {
            name: file_name,
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

    pub fn write_sea_orm_active_enums(
        &self,
        with_serde: &WithSerde,
        with_copy_enums: bool,
    ) -> OutputFile {
        let mut lines = Vec::new();
        Self::write_doc_comment(&mut lines);
        Self::write(&mut lines, vec![Self::gen_import(with_serde)]);
        lines.push("".to_owned());
        let code_blocks = self
            .enums
            .values()
            .map(|active_enum| active_enum.impl_active_enum(with_serde, with_copy_enums))
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
            "//! `SeaORM` Entity. Generated by sea-orm-codegen {ver}"
        )];
        lines.extend(comments);
        lines.push("".to_owned());
    }

    #[allow(clippy::too_many_arguments)]
    pub fn gen_expanded_code_blocks(
        entity: &Entity,
        with_serde: &WithSerde,
        date_time_crate: &DateTimeCrate,
        schema_name: &Option<String>,
        serde_skip_deserializing_primary_key: bool,
        serde_skip_hidden_column: bool,
        model_extra_derives: &TokenStream,
        model_extra_attributes: &TokenStream,
        seaography: bool,
    ) -> Vec<TokenStream> {
        let mut imports = Self::gen_import(with_serde);
        imports.extend(Self::gen_import_active_enum(entity));
        let mut code_blocks = vec![
            imports,
            Self::gen_entity_struct(),
            Self::gen_impl_entity_name(entity, schema_name),
            Self::gen_model_struct(
                entity,
                with_serde,
                date_time_crate,
                serde_skip_deserializing_primary_key,
                serde_skip_hidden_column,
                model_extra_derives,
                model_extra_attributes,
            ),
            Self::gen_column_enum(entity),
            Self::gen_primary_key_enum(entity),
            Self::gen_impl_primary_key(entity, date_time_crate),
            Self::gen_relation_enum(entity),
            Self::gen_impl_column_trait(entity),
            Self::gen_impl_relation_trait(entity),
        ];
        code_blocks.extend(Self::gen_impl_related(entity));
        code_blocks.extend(Self::gen_impl_conjunct_related(entity));
        code_blocks.extend([Self::gen_impl_active_model_behavior()]);
        if seaography {
            code_blocks.extend([Self::gen_related(entity)]);
            code_blocks.extend([Self::gen_seaography_relations(entity)]);
        }
        code_blocks
    }

    #[allow(clippy::too_many_arguments)]
    pub fn gen_compact_code_blocks(
        entity: &Entity,
        with_serde: &WithSerde,
        date_time_crate: &DateTimeCrate,
        schema_name: &Option<String>,
        serde_skip_deserializing_primary_key: bool,
        serde_skip_hidden_column: bool,
        model_extra_derives: &TokenStream,
        model_extra_attributes: &TokenStream,
        seaography: bool,
    ) -> Vec<TokenStream> {
        let mut imports = Self::gen_import(with_serde);
        imports.extend(Self::gen_import_active_enum(entity));
        let mut code_blocks = vec![
            imports,
            Self::gen_compact_model_struct(
                entity,
                with_serde,
                date_time_crate,
                schema_name,
                serde_skip_deserializing_primary_key,
                serde_skip_hidden_column,
                model_extra_derives,
                model_extra_attributes,
            ),
            Self::gen_compact_relation_enum(entity),
        ];
        code_blocks.extend([Self::gen_related_compact(entity)]);
        code_blocks.extend([Self::gen_impl_active_model_behavior()]);
        if seaography {
            code_blocks.extend([Self::gen_seaography_relations(entity)]);
        }
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

    pub fn gen_impl_entity_name(entity: &Entity, schema_name: &Option<String>) -> TokenStream {
        let schema_name = match Self::gen_schema_name(schema_name) {
            Some(schema_name) => quote! {
                fn schema_name(&self) -> Option<&str> {
                    Some(#schema_name)
                }
            },
            None => quote! {},
        };
        let table_name = entity.table_name.as_str();
        let table_name = quote! {
            fn table_name(&self) -> &str {
                #table_name
            }
        };
        quote! {
            impl EntityName for Entity {
                #schema_name
                #table_name
            }
        }
    }

    pub fn gen_import_active_enum(entity: &Entity) -> TokenStream {
        entity
            .columns
            .iter()
            .fold(TokenStream::new(), |mut ts, col| {
                if let sea_query::ColumnType::Enum { name, .. } = &col.col_type {
                    let enum_name = format_ident!("{}", name.to_string().to_upper_camel_case());
                    ts.extend([quote! {
                        use super::sea_orm_active_enums::#enum_name;
                    }]);
                }
                ts
            })
    }

    pub fn gen_model_struct(
        entity: &Entity,
        with_serde: &WithSerde,
        date_time_crate: &DateTimeCrate,
        serde_skip_deserializing_primary_key: bool,
        serde_skip_hidden_column: bool,
        model_extra_derives: &TokenStream,
        model_extra_attributes: &TokenStream,
    ) -> TokenStream {
        let column_names_snake_case = entity.get_column_names_snake_case();
        let column_rs_types = entity.get_column_rs_types(date_time_crate);
        let if_eq_needed = entity.get_eq_needed();
        let serde_attributes = entity.get_column_serde_attributes(
            serde_skip_deserializing_primary_key,
            serde_skip_hidden_column,
        );
        let extra_derive = with_serde.extra_derive();

        quote! {
            #[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel #if_eq_needed #extra_derive #model_extra_derives)]
            #model_extra_attributes
            pub struct Model {
                #(
                    #serde_attributes
                    pub #column_names_snake_case: #column_rs_types,
                )*
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

    pub fn gen_impl_primary_key(entity: &Entity, date_time_crate: &DateTimeCrate) -> TokenStream {
        let primary_key_auto_increment = entity.get_primary_key_auto_increment();
        let value_type = entity.get_primary_key_rs_type(date_time_crate);
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
            .filter(|rel| !rel.self_referencing && rel.num_suffix == 0 && rel.impl_related)
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

    pub fn gen_related(entity: &Entity) -> TokenStream {
        let related_enum_name = entity.get_related_enum_name();

        quote! {
            #[derive(Copy, Clone, Debug, EnumIter)]
            pub enum RelatedEntity {
                #(#related_enum_name),*
            }
        }
    }

    pub fn gen_related_compact(entity: &Entity) -> TokenStream {
        let related_enum_name = entity.get_related_enum_name();
        let related_attrs = entity.get_related_attrs();

        quote! {
            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
            pub enum RelatedEntity {
                #(
                    #related_attrs
                    #related_enum_name
                ),*
            }
        }
    }

    pub fn gen_impl_conjunct_related(entity: &Entity) -> Vec<TokenStream> {
        let table_name_camel_case = entity.get_table_name_camel_case_ident();
        let via_snake_case = entity.get_conjunct_relations_via_snake_case();
        let to_snake_case = entity.get_conjunct_relations_to_snake_case();
        let to_upper_camel_case = entity.get_conjunct_relations_to_upper_camel_case();
        via_snake_case
            .into_iter()
            .zip(to_snake_case)
            .zip(to_upper_camel_case)
            .map(|((via_snake_case, to_snake_case), to_upper_camel_case)| {
                quote! {
                    impl Related<super::#to_snake_case::Entity> for Entity {
                        fn to() -> RelationDef {
                            super::#via_snake_case::Relation::#to_upper_camel_case.def()
                        }

                        fn via() -> Option<RelationDef> {
                            Some(super::#via_snake_case::Relation::#table_name_camel_case.def().rev())
                        }
                    }
                }
            })
            .collect()
    }

    pub fn gen_seaography_relations(entity: &Entity) -> TokenStream {
        let basic_relations = entity.get_basic_relations();

        let related_relations = entity.get_related_relations();

        quote! {
            impl seaography::RelationBuilder for Relation {
                fn get_relation(&self, context: &'static seaography::BuilderContext) -> async_graphql::dynamic::Field {
                    let builder = seaography::EntityObjectRelationBuilder { context };
                    match self {
                        #(#basic_relations),*
                    }
                }
            }

            impl seaography::RelationBuilder for RelatedEntity {
                fn get_relation(&self, context: &'static seaography::BuilderContext) -> async_graphql::dynamic::Field {
                    let builder = seaography::EntityObjectViaRelationBuilder { context };
                    match self {
                        #(#related_relations),*
                    }
                }
            }
        }
    }

    pub fn gen_impl_active_model_behavior() -> TokenStream {
        quote! {
            impl ActiveModelBehavior for ActiveModel {}
        }
    }

    pub fn gen_mod(entity: &Entity) -> TokenStream {
        let table_name_snake_case_ident = format_ident!(
            "{}",
            escape_rust_keyword(entity.get_table_name_snake_case_ident())
        );
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

    #[allow(clippy::too_many_arguments)]
    pub fn gen_compact_model_struct(
        entity: &Entity,
        with_serde: &WithSerde,
        date_time_crate: &DateTimeCrate,
        schema_name: &Option<String>,
        serde_skip_deserializing_primary_key: bool,
        serde_skip_hidden_column: bool,
        model_extra_derives: &TokenStream,
        model_extra_attributes: &TokenStream,
    ) -> TokenStream {
        let table_name = entity.table_name.as_str();
        let column_names_snake_case = entity.get_column_names_snake_case();
        let column_rs_types = entity.get_column_rs_types(date_time_crate);
        let if_eq_needed = entity.get_eq_needed();
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
                let is_primary_key = primary_keys.contains(&col.name);
                if !col.is_snake_case_name() {
                    let column_name = &col.name;
                    attrs.push(quote! { column_name = #column_name });
                }
                if is_primary_key {
                    attrs.push(quote! { primary_key });
                    if !col.auto_increment {
                        attrs.push(quote! { auto_increment = false });
                    }
                }
                if let Some(ts) = col.get_col_type_attrs() {
                    attrs.extend([ts]);
                    if !col.not_null {
                        attrs.push(quote! { nullable });
                    }
                };
                if col.unique {
                    attrs.push(quote! { unique });
                }
                let mut ts = quote! {};
                if !attrs.is_empty() {
                    for (i, attr) in attrs.into_iter().enumerate() {
                        if i > 0 {
                            ts = quote! { #ts, };
                        }
                        ts = quote! { #ts #attr };
                    }
                    ts = quote! { #[sea_orm(#ts)] };
                }
                let serde_attribute = col.get_serde_attribute(
                    is_primary_key,
                    serde_skip_deserializing_primary_key,
                    serde_skip_hidden_column,
                );
                ts = quote! {
                    #ts
                    #serde_attribute
                };
                ts
            })
            .collect();
        let schema_name = match Self::gen_schema_name(schema_name) {
            Some(schema_name) => quote! {
                schema_name = #schema_name,
            },
            None => quote! {},
        };
        let extra_derive = with_serde.extra_derive();

        quote! {
            #[derive(Clone, Debug, PartialEq, DeriveEntityModel #if_eq_needed #extra_derive #model_extra_derives)]
            #[sea_orm(
                #schema_name
                table_name = #table_name
            )]
            #model_extra_attributes
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

    pub fn gen_schema_name(schema_name: &Option<String>) -> Option<TokenStream> {
        match schema_name {
            Some(schema_name) => {
                if schema_name != "public" {
                    Some(quote! { #schema_name })
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        entity::writer::{bonus_attributes, bonus_derive},
        Column, ConjunctRelation, DateTimeCrate, Entity, EntityWriter, PrimaryKey, Relation,
        RelationType, WithSerde,
    };
    use pretty_assertions::assert_eq;
    use proc_macro2::TokenStream;
    use sea_query::{ColumnType, ForeignKeyAction, SeaRc};
    use std::io::{self, BufRead, BufReader, Read};

    fn setup() -> Vec<Entity> {
        vec![
            Entity {
                table_name: "cake".to_owned(),
                columns: vec![
                    Column {
                        name: "id".to_owned(),
                        col_type: ColumnType::Integer,
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
                    impl_related: true,
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
                        col_type: ColumnType::Integer,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "filling_id".to_owned(),
                        col_type: ColumnType::Integer,
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
                        impl_related: true,
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
                        impl_related: true,
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
                        col_type: ColumnType::Integer,
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
                        col_type: ColumnType::Integer,
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
                        col_type: ColumnType::Integer,
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
                        impl_related: true,
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
                        impl_related: true,
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
                        col_type: ColumnType::Integer,
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
                        col_type: ColumnType::Integer,
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
                    impl_related: true,
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
                        col_type: ColumnType::Integer,
                        auto_increment: true,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "testing".to_owned(),
                        col_type: ColumnType::TinyInteger,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "rust".to_owned(),
                        col_type: ColumnType::TinyUnsigned,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "keywords".to_owned(),
                        col_type: ColumnType::SmallInteger,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "type".to_owned(),
                        col_type: ColumnType::SmallUnsigned,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "typeof".to_owned(),
                        col_type: ColumnType::Integer,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "crate".to_owned(),
                        col_type: ColumnType::Unsigned,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "self".to_owned(),
                        col_type: ColumnType::BigInteger,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "self_id1".to_owned(),
                        col_type: ColumnType::BigUnsigned,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "self_id2".to_owned(),
                        col_type: ColumnType::Integer,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "fruit_id1".to_owned(),
                        col_type: ColumnType::Integer,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "fruit_id2".to_owned(),
                        col_type: ColumnType::Integer,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "cake_id".to_owned(),
                        col_type: ColumnType::Integer,
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
                        impl_related: true,
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
                        impl_related: true,
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
                        impl_related: true,
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
                        impl_related: true,
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
                        impl_related: true,
                    },
                ],
                conjunct_relations: vec![],
                primary_keys: vec![PrimaryKey {
                    name: "id".to_owned(),
                }],
            },
            Entity {
                table_name: "cake_with_float".to_owned(),
                columns: vec![
                    Column {
                        name: "id".to_owned(),
                        col_type: ColumnType::Integer,
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
                    Column {
                        name: "price".to_owned(),
                        col_type: ColumnType::Float,
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
                    impl_related: true,
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
                table_name: "cake_with_double".to_owned(),
                columns: vec![
                    Column {
                        name: "id".to_owned(),
                        col_type: ColumnType::Integer,
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
                    Column {
                        name: "price".to_owned(),
                        col_type: ColumnType::Double,
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
                    impl_related: true,
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
                table_name: "collection".to_owned(),
                columns: vec![
                    Column {
                        name: "id".to_owned(),
                        col_type: ColumnType::Integer,
                        auto_increment: true,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "integers".to_owned(),
                        col_type: ColumnType::Array(SeaRc::new(ColumnType::Integer)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "integers_opt".to_owned(),
                        col_type: ColumnType::Array(SeaRc::new(ColumnType::Integer)),
                        auto_increment: false,
                        not_null: false,
                        unique: false,
                    },
                ],
                relations: vec![],
                conjunct_relations: vec![],
                primary_keys: vec![PrimaryKey {
                    name: "id".to_owned(),
                }],
            },
            Entity {
                table_name: "collection_float".to_owned(),
                columns: vec![
                    Column {
                        name: "id".to_owned(),
                        col_type: ColumnType::Integer,
                        auto_increment: true,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "floats".to_owned(),
                        col_type: ColumnType::Array(SeaRc::new(ColumnType::Float)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "doubles".to_owned(),
                        col_type: ColumnType::Array(SeaRc::new(ColumnType::Double)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                ],
                relations: vec![],
                conjunct_relations: vec![],
                primary_keys: vec![PrimaryKey {
                    name: "id".to_owned(),
                }],
            },
        ]
    }

    fn parse_from_file<R>(inner: R) -> io::Result<TokenStream>
    where
        R: Read,
    {
        let mut reader = BufReader::new(inner);
        let mut lines: Vec<String> = Vec::new();

        reader.read_until(b';', &mut Vec::new())?;

        let mut line = String::new();
        while reader.read_line(&mut line)? > 0 {
            lines.push(line.to_owned());
            line.clear();
        }
        let content = lines.join("");
        Ok(content.parse().unwrap())
    }

    #[test]
    fn test_gen_expanded_code_blocks() -> io::Result<()> {
        let entities = setup();
        const ENTITY_FILES: [&str; 10] = [
            include_str!("../../tests/expanded/cake.rs"),
            include_str!("../../tests/expanded/cake_filling.rs"),
            include_str!("../../tests/expanded/filling.rs"),
            include_str!("../../tests/expanded/fruit.rs"),
            include_str!("../../tests/expanded/vendor.rs"),
            include_str!("../../tests/expanded/rust_keyword.rs"),
            include_str!("../../tests/expanded/cake_with_float.rs"),
            include_str!("../../tests/expanded/cake_with_double.rs"),
            include_str!("../../tests/expanded/collection.rs"),
            include_str!("../../tests/expanded/collection_float.rs"),
        ];
        const ENTITY_FILES_WITH_SCHEMA_NAME: [&str; 10] = [
            include_str!("../../tests/expanded_with_schema_name/cake.rs"),
            include_str!("../../tests/expanded_with_schema_name/cake_filling.rs"),
            include_str!("../../tests/expanded_with_schema_name/filling.rs"),
            include_str!("../../tests/expanded_with_schema_name/fruit.rs"),
            include_str!("../../tests/expanded_with_schema_name/vendor.rs"),
            include_str!("../../tests/expanded_with_schema_name/rust_keyword.rs"),
            include_str!("../../tests/expanded_with_schema_name/cake_with_float.rs"),
            include_str!("../../tests/expanded_with_schema_name/cake_with_double.rs"),
            include_str!("../../tests/expanded_with_schema_name/collection.rs"),
            include_str!("../../tests/expanded_with_schema_name/collection_float.rs"),
        ];

        assert_eq!(entities.len(), ENTITY_FILES.len());

        for (i, entity) in entities.iter().enumerate() {
            assert_eq!(
                parse_from_file(ENTITY_FILES[i].as_bytes())?.to_string(),
                EntityWriter::gen_expanded_code_blocks(
                    entity,
                    &crate::WithSerde::None,
                    &crate::DateTimeCrate::Chrono,
                    &None,
                    false,
                    false,
                    &TokenStream::new(),
                    &TokenStream::new(),
                    false
                )
                .into_iter()
                .skip(1)
                .fold(TokenStream::new(), |mut acc, tok| {
                    acc.extend(tok);
                    acc
                })
                .to_string()
            );
            assert_eq!(
                parse_from_file(ENTITY_FILES[i].as_bytes())?.to_string(),
                EntityWriter::gen_expanded_code_blocks(
                    entity,
                    &crate::WithSerde::None,
                    &crate::DateTimeCrate::Chrono,
                    &Some("public".to_owned()),
                    false,
                    false,
                    &TokenStream::new(),
                    &TokenStream::new(),
                    false,
                )
                .into_iter()
                .skip(1)
                .fold(TokenStream::new(), |mut acc, tok| {
                    acc.extend(tok);
                    acc
                })
                .to_string()
            );
            assert_eq!(
                parse_from_file(ENTITY_FILES_WITH_SCHEMA_NAME[i].as_bytes())?.to_string(),
                EntityWriter::gen_expanded_code_blocks(
                    entity,
                    &crate::WithSerde::None,
                    &crate::DateTimeCrate::Chrono,
                    &Some("schema_name".to_owned()),
                    false,
                    false,
                    &TokenStream::new(),
                    &TokenStream::new(),
                    false,
                )
                .into_iter()
                .skip(1)
                .fold(TokenStream::new(), |mut acc, tok| {
                    acc.extend(tok);
                    acc
                })
                .to_string()
            );
        }

        Ok(())
    }

    #[test]
    fn test_gen_compact_code_blocks() -> io::Result<()> {
        let entities = setup();
        const ENTITY_FILES: [&str; 10] = [
            include_str!("../../tests/compact/cake.rs"),
            include_str!("../../tests/compact/cake_filling.rs"),
            include_str!("../../tests/compact/filling.rs"),
            include_str!("../../tests/compact/fruit.rs"),
            include_str!("../../tests/compact/vendor.rs"),
            include_str!("../../tests/compact/rust_keyword.rs"),
            include_str!("../../tests/compact/cake_with_float.rs"),
            include_str!("../../tests/compact/cake_with_double.rs"),
            include_str!("../../tests/compact/collection.rs"),
            include_str!("../../tests/compact/collection_float.rs"),
        ];
        const ENTITY_FILES_WITH_SCHEMA_NAME: [&str; 10] = [
            include_str!("../../tests/compact_with_schema_name/cake.rs"),
            include_str!("../../tests/compact_with_schema_name/cake_filling.rs"),
            include_str!("../../tests/compact_with_schema_name/filling.rs"),
            include_str!("../../tests/compact_with_schema_name/fruit.rs"),
            include_str!("../../tests/compact_with_schema_name/vendor.rs"),
            include_str!("../../tests/compact_with_schema_name/rust_keyword.rs"),
            include_str!("../../tests/compact_with_schema_name/cake_with_float.rs"),
            include_str!("../../tests/compact_with_schema_name/cake_with_double.rs"),
            include_str!("../../tests/compact_with_schema_name/collection.rs"),
            include_str!("../../tests/compact_with_schema_name/collection_float.rs"),
        ];

        assert_eq!(entities.len(), ENTITY_FILES.len());

        for (i, entity) in entities.iter().enumerate() {
            assert_eq!(
                parse_from_file(ENTITY_FILES[i].as_bytes())?.to_string(),
                EntityWriter::gen_compact_code_blocks(
                    entity,
                    &crate::WithSerde::None,
                    &crate::DateTimeCrate::Chrono,
                    &None,
                    false,
                    false,
                    &TokenStream::new(),
                    &TokenStream::new(),
                    false,
                )
                .into_iter()
                .skip(1)
                .fold(TokenStream::new(), |mut acc, tok| {
                    acc.extend(tok);
                    acc
                })
                .to_string()
            );
            assert_eq!(
                parse_from_file(ENTITY_FILES[i].as_bytes())?.to_string(),
                EntityWriter::gen_compact_code_blocks(
                    entity,
                    &crate::WithSerde::None,
                    &crate::DateTimeCrate::Chrono,
                    &Some("public".to_owned()),
                    false,
                    false,
                    &TokenStream::new(),
                    &TokenStream::new(),
                    false,
                )
                .into_iter()
                .skip(1)
                .fold(TokenStream::new(), |mut acc, tok| {
                    acc.extend(tok);
                    acc
                })
                .to_string()
            );
            assert_eq!(
                parse_from_file(ENTITY_FILES_WITH_SCHEMA_NAME[i].as_bytes())?.to_string(),
                EntityWriter::gen_compact_code_blocks(
                    entity,
                    &crate::WithSerde::None,
                    &crate::DateTimeCrate::Chrono,
                    &Some("schema_name".to_owned()),
                    false,
                    false,
                    &TokenStream::new(),
                    &TokenStream::new(),
                    false,
                )
                .into_iter()
                .skip(1)
                .fold(TokenStream::new(), |mut acc, tok| {
                    acc.extend(tok);
                    acc
                })
                .to_string()
            );
        }

        Ok(())
    }

    #[test]
    fn test_gen_with_serde() -> io::Result<()> {
        let cake_entity = setup().get(0).unwrap().clone();

        assert_eq!(cake_entity.get_table_name_snake_case(), "cake");

        // Compact code blocks
        assert_eq!(
            comparable_file_string(include_str!("../../tests/compact_with_serde/cake_none.rs"))?,
            generated_to_string(EntityWriter::gen_compact_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &DateTimeCrate::Chrono,
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                false,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/compact_with_serde/cake_serialize.rs"
            ))?,
            generated_to_string(EntityWriter::gen_compact_code_blocks(
                &cake_entity,
                &WithSerde::Serialize,
                &DateTimeCrate::Chrono,
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                false,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/compact_with_serde/cake_deserialize.rs"
            ))?,
            generated_to_string(EntityWriter::gen_compact_code_blocks(
                &cake_entity,
                &WithSerde::Deserialize,
                &DateTimeCrate::Chrono,
                &None,
                true,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                false,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!("../../tests/compact_with_serde/cake_both.rs"))?,
            generated_to_string(EntityWriter::gen_compact_code_blocks(
                &cake_entity,
                &WithSerde::Both,
                &DateTimeCrate::Chrono,
                &None,
                true,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                false,
            ))
        );

        // Expanded code blocks
        assert_eq!(
            comparable_file_string(include_str!("../../tests/expanded_with_serde/cake_none.rs"))?,
            generated_to_string(EntityWriter::gen_expanded_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &DateTimeCrate::Chrono,
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                false,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/expanded_with_serde/cake_serialize.rs"
            ))?,
            generated_to_string(EntityWriter::gen_expanded_code_blocks(
                &cake_entity,
                &WithSerde::Serialize,
                &DateTimeCrate::Chrono,
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                false,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/expanded_with_serde/cake_deserialize.rs"
            ))?,
            generated_to_string(EntityWriter::gen_expanded_code_blocks(
                &cake_entity,
                &WithSerde::Deserialize,
                &DateTimeCrate::Chrono,
                &None,
                true,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                false,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!("../../tests/expanded_with_serde/cake_both.rs"))?,
            generated_to_string(EntityWriter::gen_expanded_code_blocks(
                &cake_entity,
                &WithSerde::Both,
                &DateTimeCrate::Chrono,
                &None,
                true,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                false,
            ))
        );

        Ok(())
    }

    #[test]
    fn test_gen_with_seaography() -> io::Result<()> {
        let cake_entity = setup().get(0).unwrap().clone();

        assert_eq!(cake_entity.get_table_name_snake_case(), "cake");

        // Compact code blocks
        assert_eq!(
            comparable_file_string(include_str!("../../tests/with_seaography/cake.rs"))?,
            generated_to_string(EntityWriter::gen_compact_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &DateTimeCrate::Chrono,
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                true,
            ))
        );

        // Expanded code blocks
        assert_eq!(
            comparable_file_string(include_str!("../../tests/with_seaography/cake_expanded.rs"))?,
            generated_to_string(EntityWriter::gen_expanded_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &DateTimeCrate::Chrono,
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                true,
            ))
        );

        Ok(())
    }

    #[test]
    fn test_gen_with_derives() -> io::Result<()> {
        let mut cake_entity = setup().get_mut(0).unwrap().clone();

        assert_eq!(cake_entity.get_table_name_snake_case(), "cake");

        // Compact code blocks
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/compact_with_derives/cake_none.rs"
            ))?,
            generated_to_string(EntityWriter::gen_compact_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &DateTimeCrate::Chrono,
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                false,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!("../../tests/compact_with_derives/cake_one.rs"))?,
            generated_to_string(EntityWriter::gen_compact_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &DateTimeCrate::Chrono,
                &None,
                false,
                false,
                &bonus_derive(["ts_rs::TS"]),
                &TokenStream::new(),
                false,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/compact_with_derives/cake_multiple.rs"
            ))?,
            generated_to_string(EntityWriter::gen_compact_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &DateTimeCrate::Chrono,
                &None,
                false,
                false,
                &bonus_derive(["ts_rs::TS", "utoipa::ToSchema"]),
                &TokenStream::new(),
                false,
            ))
        );

        // Expanded code blocks
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/expanded_with_derives/cake_none.rs"
            ))?,
            generated_to_string(EntityWriter::gen_expanded_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &DateTimeCrate::Chrono,
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                false,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/expanded_with_derives/cake_one.rs"
            ))?,
            generated_to_string(EntityWriter::gen_expanded_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &DateTimeCrate::Chrono,
                &None,
                false,
                false,
                &bonus_derive(["ts_rs::TS"]),
                &TokenStream::new(),
                false,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/expanded_with_derives/cake_multiple.rs"
            ))?,
            generated_to_string(EntityWriter::gen_expanded_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &DateTimeCrate::Chrono,
                &None,
                false,
                false,
                &bonus_derive(["ts_rs::TS", "utoipa::ToSchema"]),
                &TokenStream::new(),
                false,
            ))
        );

        // Make the `name` column of `cake` entity as hidden column
        cake_entity.columns[1].name = "_name".into();

        assert_serde_variant_results(
            &cake_entity,
            &(
                include_str!("../../tests/compact_with_serde/cake_serialize_with_hidden_column.rs"),
                WithSerde::Serialize,
                None,
            ),
            Box::new(EntityWriter::gen_compact_code_blocks),
        )?;
        assert_serde_variant_results(
            &cake_entity,
            &(
                include_str!(
                    "../../tests/expanded_with_serde/cake_serialize_with_hidden_column.rs"
                ),
                WithSerde::Serialize,
                None,
            ),
            Box::new(EntityWriter::gen_expanded_code_blocks),
        )?;

        Ok(())
    }

    #[allow(clippy::type_complexity)]
    fn assert_serde_variant_results(
        cake_entity: &Entity,
        entity_serde_variant: &(&str, WithSerde, Option<String>),
        generator: Box<
            dyn Fn(
                &Entity,
                &WithSerde,
                &DateTimeCrate,
                &Option<String>,
                bool,
                bool,
                &TokenStream,
                &TokenStream,
                bool,
            ) -> Vec<TokenStream>,
        >,
    ) -> io::Result<()> {
        let mut reader = BufReader::new(entity_serde_variant.0.as_bytes());
        let mut lines: Vec<String> = Vec::new();
        let serde_skip_deserializing_primary_key = matches!(
            entity_serde_variant.1,
            WithSerde::Both | WithSerde::Deserialize
        );
        let serde_skip_hidden_column = matches!(entity_serde_variant.1, WithSerde::Serialize);

        reader.read_until(b'\n', &mut Vec::new())?;

        let mut line = String::new();
        while reader.read_line(&mut line)? > 0 {
            lines.push(line.to_owned());
            line.clear();
        }
        let content = lines.join("");
        let expected: TokenStream = content.parse().unwrap();
        println!("{:?}", entity_serde_variant.1);
        let generated = generator(
            cake_entity,
            &entity_serde_variant.1,
            &DateTimeCrate::Chrono,
            &entity_serde_variant.2,
            serde_skip_deserializing_primary_key,
            serde_skip_hidden_column,
            &TokenStream::new(),
            &TokenStream::new(),
            false,
        )
        .into_iter()
        .fold(TokenStream::new(), |mut acc, tok| {
            acc.extend(tok);
            acc
        });

        assert_eq!(expected.to_string(), generated.to_string());
        Ok(())
    }

    #[test]
    fn test_gen_with_attributes() -> io::Result<()> {
        let cake_entity = setup().get(0).unwrap().clone();

        assert_eq!(cake_entity.get_table_name_snake_case(), "cake");

        // Compact code blocks
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/compact_with_attributes/cake_none.rs"
            ))?,
            generated_to_string(EntityWriter::gen_compact_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &DateTimeCrate::Chrono,
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                false,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/compact_with_attributes/cake_one.rs"
            ))?,
            generated_to_string(EntityWriter::gen_compact_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &DateTimeCrate::Chrono,
                &None,
                false,
                false,
                &TokenStream::new(),
                &bonus_attributes([r#"serde(rename_all = "camelCase")"#]),
                false,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/compact_with_attributes/cake_multiple.rs"
            ))?,
            generated_to_string(EntityWriter::gen_compact_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &DateTimeCrate::Chrono,
                &None,
                false,
                false,
                &TokenStream::new(),
                &bonus_attributes([r#"serde(rename_all = "camelCase")"#, "ts(export)"]),
                false,
            ))
        );

        // Expanded code blocks
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/expanded_with_attributes/cake_none.rs"
            ))?,
            generated_to_string(EntityWriter::gen_expanded_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &DateTimeCrate::Chrono,
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                false,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/expanded_with_attributes/cake_one.rs"
            ))?,
            generated_to_string(EntityWriter::gen_expanded_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &DateTimeCrate::Chrono,
                &None,
                false,
                false,
                &TokenStream::new(),
                &bonus_attributes([r#"serde(rename_all = "camelCase")"#]),
                false,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/expanded_with_attributes/cake_multiple.rs"
            ))?,
            generated_to_string(EntityWriter::gen_expanded_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &DateTimeCrate::Chrono,
                &None,
                false,
                false,
                &TokenStream::new(),
                &bonus_attributes([r#"serde(rename_all = "camelCase")"#, "ts(export)"]),
                false,
            ))
        );

        Ok(())
    }

    fn generated_to_string(generated: Vec<TokenStream>) -> String {
        generated
            .into_iter()
            .fold(TokenStream::new(), |mut acc, tok| {
                acc.extend(tok);
                acc
            })
            .to_string()
    }

    fn comparable_file_string(file: &str) -> io::Result<String> {
        let mut reader = BufReader::new(file.as_bytes());
        let mut lines: Vec<String> = Vec::new();

        reader.read_until(b'\n', &mut Vec::new())?;

        let mut line = String::new();
        while reader.read_line(&mut line)? > 0 {
            lines.push(line.to_owned());
            line.clear();
        }
        let content = lines.join("");
        let expected: TokenStream = content.parse().unwrap();

        Ok(expected.to_string())
    }

    #[test]
    fn test_gen_postgres() -> io::Result<()> {
        let entities = vec![
            // This tests that the JsonBinary column type is annotated
            // correctly in compact entity form. More information can be found
            // in this issue:
            //
            // https://github.com/SeaQL/sea-orm/issues/1344
            Entity {
                table_name: "task".to_owned(),
                columns: vec![
                    Column {
                        name: "id".to_owned(),
                        col_type: ColumnType::Integer,
                        auto_increment: true,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "payload".to_owned(),
                        col_type: ColumnType::Json,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "payload_binary".to_owned(),
                        col_type: ColumnType::JsonBinary,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                ],
                relations: vec![],
                conjunct_relations: vec![],
                primary_keys: vec![PrimaryKey {
                    name: "id".to_owned(),
                }],
            },
        ];
        const ENTITY_FILES: [&str; 1] = [include_str!("../../tests/postgres/binary_json.rs")];

        const ENTITY_FILES_EXPANDED: [&str; 1] =
            [include_str!("../../tests/postgres/binary_json_expanded.rs")];

        assert_eq!(entities.len(), ENTITY_FILES.len());

        for (i, entity) in entities.iter().enumerate() {
            assert_eq!(
                parse_from_file(ENTITY_FILES[i].as_bytes())?.to_string(),
                EntityWriter::gen_compact_code_blocks(
                    entity,
                    &crate::WithSerde::None,
                    &crate::DateTimeCrate::Chrono,
                    &None,
                    false,
                    false,
                    &TokenStream::new(),
                    &TokenStream::new(),
                    false,
                )
                .into_iter()
                .skip(1)
                .fold(TokenStream::new(), |mut acc, tok| {
                    acc.extend(tok);
                    acc
                })
                .to_string()
            );
            assert_eq!(
                parse_from_file(ENTITY_FILES[i].as_bytes())?.to_string(),
                EntityWriter::gen_compact_code_blocks(
                    entity,
                    &crate::WithSerde::None,
                    &crate::DateTimeCrate::Chrono,
                    &Some("public".to_owned()),
                    false,
                    false,
                    &TokenStream::new(),
                    &TokenStream::new(),
                    false,
                )
                .into_iter()
                .skip(1)
                .fold(TokenStream::new(), |mut acc, tok| {
                    acc.extend(tok);
                    acc
                })
                .to_string()
            );
            assert_eq!(
                parse_from_file(ENTITY_FILES_EXPANDED[i].as_bytes())?.to_string(),
                EntityWriter::gen_expanded_code_blocks(
                    entity,
                    &crate::WithSerde::None,
                    &crate::DateTimeCrate::Chrono,
                    &Some("schema_name".to_owned()),
                    false,
                    false,
                    &TokenStream::new(),
                    &TokenStream::new(),
                    false,
                )
                .into_iter()
                .skip(1)
                .fold(TokenStream::new(), |mut acc, tok| {
                    acc.extend(tok);
                    acc
                })
                .to_string()
            );
        }

        Ok(())
    }
}
