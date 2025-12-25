use crate::{ActiveEnum, ColumnOption, Entity, util::escape_rust_keyword};
use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::{collections::BTreeMap, str::FromStr};
use syn::{punctuated::Punctuated, token::Comma};
use tracing::info;

mod compact;
mod dense;
mod expanded;
mod frontend;

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

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum WithPrelude {
    #[default]
    All,
    None,
    AllAllowUnusedImports,
}

#[derive(Debug, Default, PartialEq, Eq, Copy, Clone)]
pub enum WithSerde {
    #[default]
    None,
    Serialize,
    Deserialize,
    Both,
}

#[derive(Debug, Default, PartialEq, Eq, Copy, Clone)]
pub enum DateTimeCrate {
    #[default]
    Chrono,
    Time,
}

#[derive(Debug, Default, PartialEq, Eq, Copy, Clone)]
pub enum BigIntegerType {
    #[default]
    I64,
    I32,
}

#[derive(Debug, Default, PartialEq, Eq, Copy, Clone)]
pub enum EntityFormat {
    #[default]
    Compact,
    Expanded,
    Frontend,
    Dense,
}

#[derive(Debug, Default, PartialEq, Eq, Copy, Clone)]
pub enum BannerVersion {
    Off,
    Major,
    #[default]
    Minor,
    Patch,
}

#[derive(Debug)]
pub struct EntityWriterContext {
    pub(crate) entity_format: EntityFormat,
    pub(crate) with_prelude: WithPrelude,
    pub(crate) with_serde: WithSerde,
    pub(crate) with_copy_enums: bool,
    pub(crate) date_time_crate: DateTimeCrate,
    pub(crate) big_integer_type: BigIntegerType,
    pub(crate) schema_name: Option<String>,
    pub(crate) lib: bool,
    pub(crate) serde_skip_hidden_column: bool,
    pub(crate) serde_skip_deserializing_primary_key: bool,
    pub(crate) model_extra_derives: TokenStream,
    pub(crate) model_extra_attributes: TokenStream,
    pub(crate) enum_extra_derives: TokenStream,
    pub(crate) enum_extra_attributes: TokenStream,
    pub(crate) column_extra_derives: TokenStream,
    pub(crate) seaography: bool,
    pub(crate) impl_active_model_behavior: bool,
    pub(crate) banner_version: BannerVersion,
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

/// Converts *_extra_derives argument to token stream
pub(crate) fn bonus_derive<T, I>(extra_derives: I) -> TokenStream
where
    T: Into<String>,
    I: IntoIterator<Item = T>,
{
    extra_derives.into_iter().map(Into::<String>::into).fold(
        TokenStream::default(),
        |acc, derive| {
            let tokens: TokenStream = derive.parse().unwrap();
            quote! { #acc, #tokens }
        },
    )
}

/// convert *_extra_attributes argument to token stream
pub(crate) fn bonus_attributes<T, I>(attributes: I) -> TokenStream
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

impl FromStr for WithPrelude {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "none" => Self::None,
            "all-allow-unused-imports" => Self::AllAllowUnusedImports,
            "all" => Self::All,
            v => {
                return Err(crate::Error::TransformError(format!(
                    "Unsupported enum variant '{v}'"
                )));
            }
        })
    }
}

impl FromStr for EntityFormat {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "compact" => Self::Compact,
            "expanded" => Self::Expanded,
            "frontend" => Self::Frontend,
            "dense" => Self::Dense,
            v => {
                return Err(crate::Error::TransformError(format!(
                    "Unsupported enum variant '{v}'"
                )));
            }
        })
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
                    "Unsupported enum variant '{v}'"
                )));
            }
        })
    }
}

impl EntityWriterContext {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        entity_format: EntityFormat,
        with_prelude: WithPrelude,
        with_serde: WithSerde,
        with_copy_enums: bool,
        date_time_crate: DateTimeCrate,
        big_integer_type: BigIntegerType,
        schema_name: Option<String>,
        lib: bool,
        serde_skip_deserializing_primary_key: bool,
        serde_skip_hidden_column: bool,
        model_extra_derives: Vec<String>,
        model_extra_attributes: Vec<String>,
        enum_extra_derives: Vec<String>,
        enum_extra_attributes: Vec<String>,
        column_extra_derives: Vec<String>,
        seaography: bool,
        impl_active_model_behavior: bool,
        banner_version: BannerVersion,
    ) -> Self {
        Self {
            entity_format,
            with_prelude,
            with_serde,
            with_copy_enums,
            date_time_crate,
            big_integer_type,
            schema_name,
            lib,
            serde_skip_deserializing_primary_key,
            serde_skip_hidden_column,
            model_extra_derives: bonus_derive(model_extra_derives),
            model_extra_attributes: bonus_attributes(model_extra_attributes),
            enum_extra_derives: bonus_derive(enum_extra_derives),
            enum_extra_attributes: bonus_attributes(enum_extra_attributes),
            column_extra_derives: bonus_derive(column_extra_derives),
            seaography,
            impl_active_model_behavior,
            banner_version,
        }
    }

    fn column_option(&self) -> ColumnOption {
        ColumnOption {
            date_time_crate: self.date_time_crate,
            big_integer_type: self.big_integer_type,
        }
    }
}

impl EntityWriter {
    pub fn generate(self, context: &EntityWriterContext) -> WriterOutput {
        let mut files = Vec::new();
        files.extend(self.write_entities(context));
        let with_prelude = context.with_prelude != WithPrelude::None;
        files.push(self.write_index_file(
            context.lib,
            with_prelude,
            context.seaography,
            context.banner_version,
        ));
        if with_prelude {
            files.push(self.write_prelude(
                context.with_prelude,
                context.entity_format,
                context.banner_version,
            ));
        }
        if !self.enums.is_empty() {
            files.push(self.write_sea_orm_active_enums(
                &context.with_serde,
                context.with_copy_enums,
                &context.enum_extra_derives,
                &context.enum_extra_attributes,
                context.entity_format,
                context.banner_version,
            ));
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
                    .map(|column| column.get_info(&context.column_option()))
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
                Self::write_doc_comment(&mut lines, context.banner_version);
                let code_blocks = if context.entity_format == EntityFormat::Frontend {
                    Self::gen_frontend_code_blocks(
                        entity,
                        &context.with_serde,
                        &context.column_option(),
                        &context.schema_name,
                        serde_skip_deserializing_primary_key,
                        serde_skip_hidden_column,
                        &context.model_extra_derives,
                        &context.model_extra_attributes,
                        &context.column_extra_derives,
                        context.seaography,
                        context.impl_active_model_behavior,
                    )
                } else if context.entity_format == EntityFormat::Expanded {
                    Self::gen_expanded_code_blocks(
                        entity,
                        &context.with_serde,
                        &context.column_option(),
                        &context.schema_name,
                        serde_skip_deserializing_primary_key,
                        serde_skip_hidden_column,
                        &context.model_extra_derives,
                        &context.model_extra_attributes,
                        &context.column_extra_derives,
                        context.seaography,
                        context.impl_active_model_behavior,
                    )
                } else if context.entity_format == EntityFormat::Dense {
                    Self::gen_dense_code_blocks(
                        entity,
                        &context.with_serde,
                        &context.column_option(),
                        &context.schema_name,
                        serde_skip_deserializing_primary_key,
                        serde_skip_hidden_column,
                        &context.model_extra_derives,
                        &context.model_extra_attributes,
                        &context.column_extra_derives,
                        context.seaography,
                        context.impl_active_model_behavior,
                    )
                } else {
                    Self::gen_compact_code_blocks(
                        entity,
                        &context.with_serde,
                        &context.column_option(),
                        &context.schema_name,
                        serde_skip_deserializing_primary_key,
                        serde_skip_hidden_column,
                        &context.model_extra_derives,
                        &context.model_extra_attributes,
                        &context.column_extra_derives,
                        context.seaography,
                        context.impl_active_model_behavior,
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

    pub fn write_index_file(
        &self,
        lib: bool,
        prelude: bool,
        seaography: bool,
        banner_version: BannerVersion,
    ) -> OutputFile {
        let mut lines = Vec::new();
        Self::write_doc_comment(&mut lines, banner_version);
        let code_blocks: Vec<TokenStream> = self.entities.iter().map(Self::gen_mod).collect();
        if prelude {
            Self::write(
                &mut lines,
                vec![quote! {
                    pub mod prelude;
                }],
            );
            lines.push("".to_owned());
        }
        Self::write(&mut lines, code_blocks);
        if !self.enums.is_empty() {
            Self::write(
                &mut lines,
                vec![quote! {
                    pub mod sea_orm_active_enums;
                }],
            );
        }

        if seaography {
            lines.push("".to_owned());
            let ts = Self::gen_seaography_entity_mod(&self.entities, &self.enums);
            Self::write(&mut lines, vec![ts]);
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

    pub fn write_prelude(
        &self,
        with_prelude: WithPrelude,
        entity_format: EntityFormat,
        banner_version: BannerVersion,
    ) -> OutputFile {
        let mut lines = Vec::new();
        Self::write_doc_comment(&mut lines, banner_version);
        if with_prelude == WithPrelude::AllAllowUnusedImports {
            Self::write_allow_unused_imports(&mut lines)
        }
        let code_blocks = self
            .entities
            .iter()
            .map({
                if entity_format == EntityFormat::Frontend {
                    Self::gen_prelude_use_model
                } else {
                    Self::gen_prelude_use
                }
            })
            .collect();
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
        extra_derives: &TokenStream,
        extra_attributes: &TokenStream,
        entity_format: EntityFormat,
        banner_version: BannerVersion,
    ) -> OutputFile {
        let mut lines = Vec::new();
        Self::write_doc_comment(&mut lines, banner_version);
        if entity_format == EntityFormat::Frontend {
            Self::write(&mut lines, vec![Self::gen_import_serde(with_serde)]);
        } else {
            Self::write(&mut lines, vec![Self::gen_import(with_serde)]);
        }
        lines.push("".to_owned());
        let code_blocks = self
            .enums
            .values()
            .map(|active_enum| {
                active_enum.impl_active_enum(
                    with_serde,
                    with_copy_enums,
                    extra_derives,
                    extra_attributes,
                    entity_format,
                )
            })
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

    pub fn write_doc_comment(lines: &mut Vec<String>, banner_version: BannerVersion) {
        let ver = env!("CARGO_PKG_VERSION");
        let version_str = match banner_version {
            BannerVersion::Off => String::new(),
            BannerVersion::Patch => ver.to_owned(),
            _ => {
                let parts: Vec<&str> = ver.split('.').collect();
                match banner_version {
                    BannerVersion::Major => {
                        parts.first().map(|x| (*x).to_owned()).unwrap_or_default()
                    }
                    BannerVersion::Minor => {
                        if parts.len() >= 2 {
                            format!("{}.{}", parts[0], parts[1])
                        } else {
                            ver.to_owned()
                        }
                    }
                    _ => unreachable!(),
                }
            }
        };
        let comments = vec![format!(
            "//! `SeaORM` Entity, @generated by sea-orm-codegen {version_str}"
        )];
        lines.extend(comments);
        lines.push("".to_owned());
    }

    pub fn write_allow_unused_imports(lines: &mut Vec<String>) {
        lines.extend(vec!["#![allow(unused_imports)]".to_string()]);
        lines.push("".to_owned());
    }

    pub fn gen_import(with_serde: &WithSerde) -> TokenStream {
        let serde_import = Self::gen_import_serde(with_serde);
        quote! {
            use sea_orm::entity::prelude::*;
            #serde_import
        }
    }

    pub fn gen_import_serde(with_serde: &WithSerde) -> TokenStream {
        match with_serde {
            WithSerde::None => Default::default(),
            WithSerde::Serialize => {
                quote! {
                    use serde::Serialize;
                }
            }
            WithSerde::Deserialize => {
                quote! {
                    use serde::Deserialize;
                }
            }
            WithSerde::Both => {
                quote! {
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
            .fold(
                (TokenStream::new(), Vec::new()),
                |(mut ts, mut enums), col| {
                    if let sea_query::ColumnType::Enum { name, .. } = col.get_inner_col_type() {
                        if !enums.contains(&name) {
                            enums.push(name);
                            let enum_name =
                                format_ident!("{}", name.to_string().to_upper_camel_case());
                            ts.extend([quote! {
                                use super::sea_orm_active_enums::#enum_name;
                            }]);
                        }
                    }
                    (ts, enums)
                },
            )
            .0
    }

    pub fn gen_column_enum(entity: &Entity, column_extra_derives: &TokenStream) -> TokenStream {
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
            #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn #column_extra_derives)]
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

    pub fn gen_impl_primary_key(entity: &Entity, column_option: &ColumnOption) -> TokenStream {
        let primary_key_auto_increment = entity.get_primary_key_auto_increment();
        let value_type = entity.get_primary_key_rs_type(column_option);
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

    /// Used to generate `enum RelatedEntity` that is useful to the Seaography project
    pub fn gen_related_entity(entity: &Entity) -> TokenStream {
        let related_enum_name = entity.get_related_entity_enum_name();
        let related_attrs = entity.get_related_entity_attrs();

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

    pub fn impl_active_model_behavior() -> TokenStream {
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

    pub fn gen_seaography_entity_mod(
        entities: &[Entity],
        enums: &BTreeMap<String, ActiveEnum>,
    ) -> TokenStream {
        let mut ts = TokenStream::new();
        for entity in entities {
            let table_name_snake_case_ident = format_ident!(
                "{}",
                escape_rust_keyword(entity.get_table_name_snake_case_ident())
            );
            ts = quote! {
                #ts
                #table_name_snake_case_ident,
            }
        }
        ts = quote! {
            seaography::register_entity_modules!([
                #ts
            ]);
        };

        let mut enum_ts = TokenStream::new();
        for active_enum in enums.values() {
            let enum_name = &active_enum.enum_name.to_string();
            let enum_iden = format_ident!("{}", enum_name.to_upper_camel_case());
            enum_ts = quote! {
                #enum_ts
                sea_orm_active_enums::#enum_iden,
            }
        }
        if !enum_ts.is_empty() {
            ts = quote! {
                #ts

                seaography::register_active_enums!([
                    #enum_ts
                ]);
            };
        }
        ts
    }

    pub fn gen_prelude_use(entity: &Entity) -> TokenStream {
        let table_name_snake_case_ident = entity.get_table_name_snake_case_ident();
        let table_name_camel_case_ident = entity.get_table_name_camel_case_ident();
        quote! {
            pub use super::#table_name_snake_case_ident::Entity as #table_name_camel_case_ident;
        }
    }

    pub fn gen_prelude_use_model(entity: &Entity) -> TokenStream {
        let table_name_snake_case_ident = entity.get_table_name_snake_case_ident();
        let table_name_camel_case_ident = entity.get_table_name_camel_case_ident();
        quote! {
            pub use super::#table_name_snake_case_ident::Model as #table_name_camel_case_ident;
        }
    }

    pub fn gen_schema_name(schema_name: &Option<String>) -> Option<TokenStream> {
        schema_name
            .as_ref()
            .map(|schema_name| quote! { #schema_name })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Column, ColumnOption, ConjunctRelation, Entity, EntityWriter, PrimaryKey, Relation,
        RelationType, WithSerde,
        entity::writer::{bonus_attributes, bonus_derive},
    };
    use pretty_assertions::assert_eq;
    use proc_macro2::TokenStream;
    use quote::quote;
    use sea_query::{Alias, ColumnType, ForeignKeyAction, RcOrArc, SeaRc, StringLen};
    use std::io::{self, BufRead, BufReader, Read};

    fn default_column_option() -> ColumnOption {
        Default::default()
    }

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
                        unique_key: None,
                    },
                    Column {
                        name: "name".to_owned(),
                        col_type: ColumnType::Text,
                        auto_increment: false,
                        not_null: false,
                        unique: false,
                        unique_key: None,
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
                is_view: false,
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
                        unique_key: None,
                    },
                    Column {
                        name: "filling_id".to_owned(),
                        col_type: ColumnType::Integer,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
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
                is_view: false,
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
                table_name: "cake_filling_price".to_owned(),
                columns: vec![
                    Column {
                        name: "cake_id".to_owned(),
                        col_type: ColumnType::Integer,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "filling_id".to_owned(),
                        col_type: ColumnType::Integer,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "price".to_owned(),
                        col_type: ColumnType::Decimal(None),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                ],
                relations: vec![Relation {
                    ref_table: "cake_filling".to_owned(),
                    columns: vec!["cake_id".to_owned(), "filling_id".to_owned()],
                    ref_columns: vec!["cake_id".to_owned(), "filling_id".to_owned()],
                    rel_type: RelationType::BelongsTo,
                    on_delete: None,
                    on_update: None,
                    self_referencing: false,
                    num_suffix: 0,
                    impl_related: true,
                }],
                conjunct_relations: vec![],
                is_view: false,
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
                        unique_key: None,
                    },
                    Column {
                        name: "name".to_owned(),
                        col_type: ColumnType::String(StringLen::N(255)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                ],
                relations: vec![],
                conjunct_relations: vec![ConjunctRelation {
                    via: "cake_filling".to_owned(),
                    to: "cake".to_owned(),
                }],
                is_view: false,
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
                        unique_key: None,
                    },
                    Column {
                        name: "name".to_owned(),
                        col_type: ColumnType::String(StringLen::N(255)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "cake_id".to_owned(),
                        col_type: ColumnType::Integer,
                        auto_increment: false,
                        not_null: false,
                        unique: false,
                        unique_key: None,
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
                is_view: false,
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
                        unique_key: None,
                    },
                    Column {
                        name: "_name_".to_owned(),
                        col_type: ColumnType::String(StringLen::N(255)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "fruitId".to_owned(),
                        col_type: ColumnType::Integer,
                        auto_increment: false,
                        not_null: false,
                        unique: false,
                        unique_key: None,
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
                is_view: false,
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
                        unique_key: None,
                    },
                    Column {
                        name: "testing".to_owned(),
                        col_type: ColumnType::TinyInteger,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "rust".to_owned(),
                        col_type: ColumnType::TinyUnsigned,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "keywords".to_owned(),
                        col_type: ColumnType::SmallInteger,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "type".to_owned(),
                        col_type: ColumnType::SmallUnsigned,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "typeof".to_owned(),
                        col_type: ColumnType::Integer,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "crate".to_owned(),
                        col_type: ColumnType::Unsigned,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "self".to_owned(),
                        col_type: ColumnType::BigInteger,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "self_id1".to_owned(),
                        col_type: ColumnType::BigUnsigned,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "self_id2".to_owned(),
                        col_type: ColumnType::Integer,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "fruit_id1".to_owned(),
                        col_type: ColumnType::Integer,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "fruit_id2".to_owned(),
                        col_type: ColumnType::Integer,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "cake_id".to_owned(),
                        col_type: ColumnType::Integer,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
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
                is_view: false,
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
                        unique_key: None,
                    },
                    Column {
                        name: "name".to_owned(),
                        col_type: ColumnType::Text,
                        auto_increment: false,
                        not_null: false,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "price".to_owned(),
                        col_type: ColumnType::Float,
                        auto_increment: false,
                        not_null: false,
                        unique: false,
                        unique_key: None,
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
                is_view: false,
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
                        unique_key: None,
                    },
                    Column {
                        name: "name".to_owned(),
                        col_type: ColumnType::Text,
                        auto_increment: false,
                        not_null: false,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "price".to_owned(),
                        col_type: ColumnType::Double,
                        auto_increment: false,
                        not_null: false,
                        unique: false,
                        unique_key: None,
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
                is_view: false,
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
                        unique_key: None,
                    },
                    Column {
                        name: "integers".to_owned(),
                        col_type: ColumnType::Array(RcOrArc::new(ColumnType::Integer)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "integers_opt".to_owned(),
                        col_type: ColumnType::Array(RcOrArc::new(ColumnType::Integer)),
                        auto_increment: false,
                        not_null: false,
                        unique: false,
                        unique_key: None,
                    },
                ],
                relations: vec![],
                conjunct_relations: vec![],
                is_view: false,
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
                        unique_key: None,
                    },
                    Column {
                        name: "floats".to_owned(),
                        col_type: ColumnType::Array(RcOrArc::new(ColumnType::Float)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "doubles".to_owned(),
                        col_type: ColumnType::Array(RcOrArc::new(ColumnType::Double)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                ],
                relations: vec![],
                conjunct_relations: vec![],
                is_view: false,
                primary_keys: vec![PrimaryKey {
                    name: "id".to_owned(),
                }],
            },
            Entity {
                table_name: "parent".to_owned(),
                columns: vec![
                    Column {
                        name: "id1".to_owned(),
                        col_type: ColumnType::Integer,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "id2".to_owned(),
                        col_type: ColumnType::Integer,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                ],
                relations: vec![Relation {
                    ref_table: "child".to_owned(),
                    columns: vec![],
                    ref_columns: vec![],
                    rel_type: RelationType::HasMany,
                    on_delete: None,
                    on_update: None,
                    self_referencing: false,
                    num_suffix: 0,
                    impl_related: true,
                }],
                conjunct_relations: vec![],
                is_view: false,
                primary_keys: vec![
                    PrimaryKey {
                        name: "id1".to_owned(),
                    },
                    PrimaryKey {
                        name: "id2".to_owned(),
                    },
                ],
            },
            Entity {
                table_name: "child".to_owned(),
                columns: vec![
                    Column {
                        name: "id".to_owned(),
                        col_type: ColumnType::Integer,
                        auto_increment: true,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "parent_id1".to_owned(),
                        col_type: ColumnType::Integer,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "parent_id2".to_owned(),
                        col_type: ColumnType::Integer,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                ],
                relations: vec![Relation {
                    ref_table: "parent".to_owned(),
                    columns: vec!["parent_id1".to_owned(), "parent_id2".to_owned()],
                    ref_columns: vec!["id1".to_owned(), "id2".to_owned()],
                    rel_type: RelationType::BelongsTo,
                    on_delete: None,
                    on_update: None,
                    self_referencing: false,
                    num_suffix: 0,
                    impl_related: true,
                }],
                conjunct_relations: vec![],
                is_view: false,
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

    fn parse_from_frontend_file<R>(inner: R) -> io::Result<TokenStream>
    where
        R: Read,
    {
        let mut reader = BufReader::new(inner);
        let mut lines: Vec<String> = Vec::new();

        reader.read_until(b'\n', &mut Vec::new())?;

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
        const ENTITY_FILES: [&str; 13] = [
            include_str!("../../tests/expanded/cake.rs"),
            include_str!("../../tests/expanded/cake_filling.rs"),
            include_str!("../../tests/expanded/cake_filling_price.rs"),
            include_str!("../../tests/expanded/filling.rs"),
            include_str!("../../tests/expanded/fruit.rs"),
            include_str!("../../tests/expanded/vendor.rs"),
            include_str!("../../tests/expanded/rust_keyword.rs"),
            include_str!("../../tests/expanded/cake_with_float.rs"),
            include_str!("../../tests/expanded/cake_with_double.rs"),
            include_str!("../../tests/expanded/collection.rs"),
            include_str!("../../tests/expanded/collection_float.rs"),
            include_str!("../../tests/expanded/parent.rs"),
            include_str!("../../tests/expanded/child.rs"),
        ];
        const ENTITY_FILES_WITH_SCHEMA_NAME: [&str; 13] = [
            include_str!("../../tests/expanded_with_schema_name/cake.rs"),
            include_str!("../../tests/expanded_with_schema_name/cake_filling.rs"),
            include_str!("../../tests/expanded_with_schema_name/cake_filling_price.rs"),
            include_str!("../../tests/expanded_with_schema_name/filling.rs"),
            include_str!("../../tests/expanded_with_schema_name/fruit.rs"),
            include_str!("../../tests/expanded_with_schema_name/vendor.rs"),
            include_str!("../../tests/expanded_with_schema_name/rust_keyword.rs"),
            include_str!("../../tests/expanded_with_schema_name/cake_with_float.rs"),
            include_str!("../../tests/expanded_with_schema_name/cake_with_double.rs"),
            include_str!("../../tests/expanded_with_schema_name/collection.rs"),
            include_str!("../../tests/expanded_with_schema_name/collection_float.rs"),
            include_str!("../../tests/expanded_with_schema_name/parent.rs"),
            include_str!("../../tests/expanded_with_schema_name/child.rs"),
        ];

        assert_eq!(entities.len(), ENTITY_FILES.len());

        for (i, entity) in entities.iter().enumerate() {
            assert_eq!(
                parse_from_file(ENTITY_FILES[i].as_bytes())?.to_string(),
                EntityWriter::gen_expanded_code_blocks(
                    entity,
                    &crate::WithSerde::None,
                    &default_column_option(),
                    &None,
                    false,
                    false,
                    &TokenStream::new(),
                    &TokenStream::new(),
                    &TokenStream::new(),
                    false,
                    true,
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
                    &default_column_option(),
                    &Some("schema_name".to_owned()),
                    false,
                    false,
                    &TokenStream::new(),
                    &TokenStream::new(),
                    &TokenStream::new(),
                    false,
                    true,
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
        const ENTITY_FILES: [&str; 13] = [
            include_str!("../../tests/compact/cake.rs"),
            include_str!("../../tests/compact/cake_filling.rs"),
            include_str!("../../tests/compact/cake_filling_price.rs"),
            include_str!("../../tests/compact/filling.rs"),
            include_str!("../../tests/compact/fruit.rs"),
            include_str!("../../tests/compact/vendor.rs"),
            include_str!("../../tests/compact/rust_keyword.rs"),
            include_str!("../../tests/compact/cake_with_float.rs"),
            include_str!("../../tests/compact/cake_with_double.rs"),
            include_str!("../../tests/compact/collection.rs"),
            include_str!("../../tests/compact/collection_float.rs"),
            include_str!("../../tests/compact/parent.rs"),
            include_str!("../../tests/compact/child.rs"),
        ];
        const ENTITY_FILES_WITH_SCHEMA_NAME: [&str; 13] = [
            include_str!("../../tests/compact_with_schema_name/cake.rs"),
            include_str!("../../tests/compact_with_schema_name/cake_filling.rs"),
            include_str!("../../tests/compact_with_schema_name/cake_filling_price.rs"),
            include_str!("../../tests/compact_with_schema_name/filling.rs"),
            include_str!("../../tests/compact_with_schema_name/fruit.rs"),
            include_str!("../../tests/compact_with_schema_name/vendor.rs"),
            include_str!("../../tests/compact_with_schema_name/rust_keyword.rs"),
            include_str!("../../tests/compact_with_schema_name/cake_with_float.rs"),
            include_str!("../../tests/compact_with_schema_name/cake_with_double.rs"),
            include_str!("../../tests/compact_with_schema_name/collection.rs"),
            include_str!("../../tests/compact_with_schema_name/collection_float.rs"),
            include_str!("../../tests/compact_with_schema_name/parent.rs"),
            include_str!("../../tests/compact_with_schema_name/child.rs"),
        ];

        assert_eq!(entities.len(), ENTITY_FILES.len());

        for (i, entity) in entities.iter().enumerate() {
            assert_eq!(
                parse_from_file(ENTITY_FILES[i].as_bytes())?.to_string(),
                EntityWriter::gen_compact_code_blocks(
                    entity,
                    &crate::WithSerde::None,
                    &default_column_option(),
                    &None,
                    false,
                    false,
                    &TokenStream::new(),
                    &TokenStream::new(),
                    &TokenStream::new(),
                    false,
                    true,
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
                    &default_column_option(),
                    &Some("schema_name".to_owned()),
                    false,
                    false,
                    &TokenStream::new(),
                    &TokenStream::new(),
                    &TokenStream::new(),
                    false,
                    true,
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
    fn test_gen_frontend_code_blocks() -> io::Result<()> {
        let entities = setup();
        const ENTITY_FILES: [&str; 13] = [
            include_str!("../../tests/frontend/cake.rs"),
            include_str!("../../tests/frontend/cake_filling.rs"),
            include_str!("../../tests/frontend/cake_filling_price.rs"),
            include_str!("../../tests/frontend/filling.rs"),
            include_str!("../../tests/frontend/fruit.rs"),
            include_str!("../../tests/frontend/vendor.rs"),
            include_str!("../../tests/frontend/rust_keyword.rs"),
            include_str!("../../tests/frontend/cake_with_float.rs"),
            include_str!("../../tests/frontend/cake_with_double.rs"),
            include_str!("../../tests/frontend/collection.rs"),
            include_str!("../../tests/frontend/collection_float.rs"),
            include_str!("../../tests/frontend/parent.rs"),
            include_str!("../../tests/frontend/child.rs"),
        ];
        const ENTITY_FILES_WITH_SCHEMA_NAME: [&str; 13] = [
            include_str!("../../tests/frontend_with_schema_name/cake.rs"),
            include_str!("../../tests/frontend_with_schema_name/cake_filling.rs"),
            include_str!("../../tests/frontend_with_schema_name/cake_filling_price.rs"),
            include_str!("../../tests/frontend_with_schema_name/filling.rs"),
            include_str!("../../tests/frontend_with_schema_name/fruit.rs"),
            include_str!("../../tests/frontend_with_schema_name/vendor.rs"),
            include_str!("../../tests/frontend_with_schema_name/rust_keyword.rs"),
            include_str!("../../tests/frontend_with_schema_name/cake_with_float.rs"),
            include_str!("../../tests/frontend_with_schema_name/cake_with_double.rs"),
            include_str!("../../tests/frontend_with_schema_name/collection.rs"),
            include_str!("../../tests/frontend_with_schema_name/collection_float.rs"),
            include_str!("../../tests/frontend_with_schema_name/parent.rs"),
            include_str!("../../tests/frontend_with_schema_name/child.rs"),
        ];

        assert_eq!(entities.len(), ENTITY_FILES.len());

        for (i, entity) in entities.iter().enumerate() {
            assert_eq!(
                dbg!(parse_from_frontend_file(ENTITY_FILES[i].as_bytes())?.to_string()),
                EntityWriter::gen_frontend_code_blocks(
                    entity,
                    &crate::WithSerde::None,
                    &default_column_option(),
                    &None,
                    false,
                    false,
                    &TokenStream::new(),
                    &TokenStream::new(),
                    &TokenStream::new(),
                    false,
                    true,
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
                parse_from_frontend_file(ENTITY_FILES_WITH_SCHEMA_NAME[i].as_bytes())?.to_string(),
                EntityWriter::gen_frontend_code_blocks(
                    entity,
                    &crate::WithSerde::None,
                    &default_column_option(),
                    &Some("schema_name".to_owned()),
                    false,
                    false,
                    &TokenStream::new(),
                    &TokenStream::new(),
                    &TokenStream::new(),
                    false,
                    true,
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
                &default_column_option(),
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                &TokenStream::new(),
                false,
                true,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/compact_with_serde/cake_serialize.rs"
            ))?,
            generated_to_string(EntityWriter::gen_compact_code_blocks(
                &cake_entity,
                &WithSerde::Serialize,
                &default_column_option(),
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                &TokenStream::new(),
                false,
                true,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/compact_with_serde/cake_deserialize.rs"
            ))?,
            generated_to_string(EntityWriter::gen_compact_code_blocks(
                &cake_entity,
                &WithSerde::Deserialize,
                &default_column_option(),
                &None,
                true,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                &TokenStream::new(),
                false,
                true,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!("../../tests/compact_with_serde/cake_both.rs"))?,
            generated_to_string(EntityWriter::gen_compact_code_blocks(
                &cake_entity,
                &WithSerde::Both,
                &default_column_option(),
                &None,
                true,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                &TokenStream::new(),
                false,
                true,
            ))
        );

        // Expanded code blocks
        assert_eq!(
            comparable_file_string(include_str!("../../tests/expanded_with_serde/cake_none.rs"))?,
            generated_to_string(EntityWriter::gen_expanded_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &default_column_option(),
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                &TokenStream::new(),
                false,
                true,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/expanded_with_serde/cake_serialize.rs"
            ))?,
            generated_to_string(EntityWriter::gen_expanded_code_blocks(
                &cake_entity,
                &WithSerde::Serialize,
                &default_column_option(),
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                &TokenStream::new(),
                false,
                true,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/expanded_with_serde/cake_deserialize.rs"
            ))?,
            generated_to_string(EntityWriter::gen_expanded_code_blocks(
                &cake_entity,
                &WithSerde::Deserialize,
                &default_column_option(),
                &None,
                true,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                &TokenStream::new(),
                false,
                true,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!("../../tests/expanded_with_serde/cake_both.rs"))?,
            generated_to_string(EntityWriter::gen_expanded_code_blocks(
                &cake_entity,
                &WithSerde::Both,
                &default_column_option(),
                &None,
                true,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                &TokenStream::new(),
                false,
                true,
            ))
        );

        // Frontend code blocks
        assert_eq!(
            comparable_file_string(include_str!("../../tests/frontend_with_serde/cake_none.rs"))?,
            generated_to_string(EntityWriter::gen_frontend_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &default_column_option(),
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                &TokenStream::new(),
                false,
                true,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/frontend_with_serde/cake_serialize.rs"
            ))?,
            generated_to_string(EntityWriter::gen_frontend_code_blocks(
                &cake_entity,
                &WithSerde::Serialize,
                &default_column_option(),
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                &TokenStream::new(),
                false,
                true,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/frontend_with_serde/cake_deserialize.rs"
            ))?,
            generated_to_string(EntityWriter::gen_frontend_code_blocks(
                &cake_entity,
                &WithSerde::Deserialize,
                &default_column_option(),
                &None,
                true,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                &TokenStream::new(),
                false,
                true,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!("../../tests/frontend_with_serde/cake_both.rs"))?,
            generated_to_string(EntityWriter::gen_frontend_code_blocks(
                &cake_entity,
                &WithSerde::Both,
                &default_column_option(),
                &None,
                true,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                &TokenStream::new(),
                false,
                true,
            ))
        );

        Ok(())
    }

    #[test]
    fn test_gen_with_seaography() -> io::Result<()> {
        let cake_entity = Entity {
            table_name: "cake".to_owned(),
            columns: vec![
                Column {
                    name: "id".to_owned(),
                    col_type: ColumnType::Integer,
                    auto_increment: true,
                    not_null: true,
                    unique: false,
                    unique_key: None,
                },
                Column {
                    name: "name".to_owned(),
                    col_type: ColumnType::Text,
                    auto_increment: false,
                    not_null: false,
                    unique: false,
                    unique_key: None,
                },
                Column {
                    name: "base_id".to_owned(),
                    col_type: ColumnType::Integer,
                    auto_increment: false,
                    not_null: false,
                    unique: false,
                    unique_key: None,
                },
            ],
            relations: vec![
                Relation {
                    ref_table: "fruit".to_owned(),
                    columns: vec![],
                    ref_columns: vec![],
                    rel_type: RelationType::HasMany,
                    on_delete: None,
                    on_update: None,
                    self_referencing: false,
                    num_suffix: 0,
                    impl_related: true,
                },
                Relation {
                    ref_table: "cake".to_owned(),
                    columns: vec![],
                    ref_columns: vec![],
                    rel_type: RelationType::HasOne,
                    on_delete: None,
                    on_update: None,
                    self_referencing: true,
                    num_suffix: 0,
                    impl_related: true,
                },
            ],
            conjunct_relations: vec![ConjunctRelation {
                via: "cake_filling".to_owned(),
                to: "filling".to_owned(),
            }],
            is_view: false,
            primary_keys: vec![PrimaryKey {
                name: "id".to_owned(),
            }],
        };

        assert_eq!(cake_entity.get_table_name_snake_case(), "cake");

        // Compact code blocks
        assert_eq!(
            comparable_file_string(include_str!("../../tests/with_seaography/cake.rs"))?,
            generated_to_string(EntityWriter::gen_compact_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &default_column_option(),
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                &TokenStream::new(),
                true,
                true,
            ))
        );

        // Expanded code blocks
        assert_eq!(
            comparable_file_string(include_str!("../../tests/with_seaography/cake_expanded.rs"))?,
            generated_to_string(EntityWriter::gen_expanded_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &default_column_option(),
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                &TokenStream::new(),
                true,
                true,
            ))
        );

        // Frontend code blocks
        assert_eq!(
            comparable_file_string(include_str!("../../tests/with_seaography/cake_frontend.rs"))?,
            generated_to_string(EntityWriter::gen_frontend_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &default_column_option(),
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                &TokenStream::new(),
                true,
                true,
            ))
        );

        Ok(())
    }

    #[test]
    fn test_gen_with_seaography_mod() -> io::Result<()> {
        use crate::ActiveEnum;
        use sea_query::IntoIden;

        let entities = setup();
        let enums = vec![
            (
                "coinflip_result_type",
                ActiveEnum {
                    enum_name: Alias::new("coinflip_result_type").into_iden(),
                    values: vec!["HEADS", "TAILS"]
                        .into_iter()
                        .map(|variant| Alias::new(variant).into_iden())
                        .collect(),
                },
            ),
            (
                "media_type",
                ActiveEnum {
                    enum_name: Alias::new("media_type").into_iden(),
                    values: vec![
                        "UNKNOWN",
                        "BITMAP",
                        "DRAWING",
                        "AUDIO",
                        "VIDEO",
                        "MULTIMEDIA",
                        "OFFICE",
                        "TEXT",
                        "EXECUTABLE",
                        "ARCHIVE",
                        "3D",
                    ]
                    .into_iter()
                    .map(|variant| Alias::new(variant).into_iden())
                    .collect(),
                },
            ),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();

        assert_eq!(
            comparable_file_string(include_str!("../../tests/with_seaography/mod.rs"))?,
            generated_to_string(vec![EntityWriter::gen_seaography_entity_mod(
                &entities, &enums,
            )])
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
                &default_column_option(),
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                &TokenStream::new(),
                false,
                true,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!("../../tests/compact_with_derives/cake_one.rs"))?,
            generated_to_string(EntityWriter::gen_compact_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &default_column_option(),
                &None,
                false,
                false,
                &bonus_derive(["ts_rs::TS"]),
                &TokenStream::new(),
                &TokenStream::new(),
                false,
                true,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/compact_with_derives/cake_multiple.rs"
            ))?,
            generated_to_string(EntityWriter::gen_compact_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &default_column_option(),
                &None,
                false,
                false,
                &bonus_derive(["ts_rs::TS", "utoipa::ToSchema"]),
                &TokenStream::new(),
                &TokenStream::new(),
                false,
                true,
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
                &default_column_option(),
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                &TokenStream::new(),
                false,
                true,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/expanded_with_derives/cake_one.rs"
            ))?,
            generated_to_string(EntityWriter::gen_expanded_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &default_column_option(),
                &None,
                false,
                false,
                &bonus_derive(["ts_rs::TS"]),
                &TokenStream::new(),
                &TokenStream::new(),
                false,
                true,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/expanded_with_derives/cake_multiple.rs"
            ))?,
            generated_to_string(EntityWriter::gen_expanded_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &default_column_option(),
                &None,
                false,
                false,
                &bonus_derive(["ts_rs::TS", "utoipa::ToSchema"]),
                &TokenStream::new(),
                &TokenStream::new(),
                false,
                true,
            ))
        );

        // Frontend code blocks
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/frontend_with_derives/cake_none.rs"
            ))?,
            generated_to_string(EntityWriter::gen_frontend_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &default_column_option(),
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                &TokenStream::new(),
                false,
                true,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/frontend_with_derives/cake_one.rs"
            ))?,
            generated_to_string(EntityWriter::gen_frontend_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &default_column_option(),
                &None,
                false,
                false,
                &bonus_derive(["ts_rs::TS"]),
                &TokenStream::new(),
                &TokenStream::new(),
                false,
                true,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/frontend_with_derives/cake_multiple.rs"
            ))?,
            generated_to_string(EntityWriter::gen_frontend_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &default_column_option(),
                &None,
                false,
                false,
                &bonus_derive(["ts_rs::TS", "utoipa::ToSchema"]),
                &TokenStream::new(),
                &TokenStream::new(),
                false,
                true,
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
        assert_serde_variant_results(
            &cake_entity,
            &(
                include_str!(
                    "../../tests/frontend_with_serde/cake_serialize_with_hidden_column.rs"
                ),
                WithSerde::Serialize,
                None,
            ),
            Box::new(EntityWriter::gen_frontend_code_blocks),
        )?;

        Ok(())
    }

    #[test]
    fn test_gen_with_column_derives() -> io::Result<()> {
        let cake_entity = setup().get_mut(0).unwrap().clone();

        assert_eq!(cake_entity.get_table_name_snake_case(), "cake");

        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/expanded_with_column_derives/cake_one.rs"
            ))?,
            generated_to_string(EntityWriter::gen_expanded_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &default_column_option(),
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                &bonus_derive(["async_graphql::Enum"]),
                false,
                true,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/expanded_with_column_derives/cake_multiple.rs"
            ))?,
            generated_to_string(EntityWriter::gen_expanded_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &default_column_option(),
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                &bonus_derive(["async_graphql::Enum", "Eq", "PartialEq"]),
                false,
                true,
            ))
        );

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
                &ColumnOption,
                &Option<String>,
                bool,
                bool,
                &TokenStream,
                &TokenStream,
                &TokenStream,
                bool,
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
            &default_column_option(),
            &entity_serde_variant.2,
            serde_skip_deserializing_primary_key,
            serde_skip_hidden_column,
            &TokenStream::new(),
            &TokenStream::new(),
            &TokenStream::new(),
            false,
            true,
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
                &default_column_option(),
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                &TokenStream::new(),
                false,
                true,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/compact_with_attributes/cake_one.rs"
            ))?,
            generated_to_string(EntityWriter::gen_compact_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &default_column_option(),
                &None,
                false,
                false,
                &TokenStream::new(),
                &bonus_attributes([r#"serde(rename_all = "camelCase")"#]),
                &TokenStream::new(),
                false,
                true,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/compact_with_attributes/cake_multiple.rs"
            ))?,
            generated_to_string(EntityWriter::gen_compact_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &default_column_option(),
                &None,
                false,
                false,
                &TokenStream::new(),
                &bonus_attributes([r#"serde(rename_all = "camelCase")"#, "ts(export)"]),
                &TokenStream::new(),
                false,
                true,
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
                &default_column_option(),
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                &TokenStream::new(),
                false,
                true,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/expanded_with_attributes/cake_one.rs"
            ))?,
            generated_to_string(EntityWriter::gen_expanded_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &default_column_option(),
                &None,
                false,
                false,
                &TokenStream::new(),
                &bonus_attributes([r#"serde(rename_all = "camelCase")"#]),
                &TokenStream::new(),
                false,
                true,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/expanded_with_attributes/cake_multiple.rs"
            ))?,
            generated_to_string(EntityWriter::gen_expanded_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &default_column_option(),
                &None,
                false,
                false,
                &TokenStream::new(),
                &bonus_attributes([r#"serde(rename_all = "camelCase")"#, "ts(export)"]),
                &TokenStream::new(),
                false,
                true,
            ))
        );

        // Frontend code blocks
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/frontend_with_attributes/cake_none.rs"
            ))?,
            generated_to_string(EntityWriter::gen_frontend_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &default_column_option(),
                &None,
                false,
                false,
                &TokenStream::new(),
                &TokenStream::new(),
                &TokenStream::new(),
                false,
                true,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/frontend_with_attributes/cake_one.rs"
            ))?,
            generated_to_string(EntityWriter::gen_frontend_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &default_column_option(),
                &None,
                false,
                false,
                &TokenStream::new(),
                &bonus_attributes([r#"serde(rename_all = "camelCase")"#]),
                &TokenStream::new(),
                false,
                true,
            ))
        );
        assert_eq!(
            comparable_file_string(include_str!(
                "../../tests/frontend_with_attributes/cake_multiple.rs"
            ))?,
            generated_to_string(EntityWriter::gen_frontend_code_blocks(
                &cake_entity,
                &WithSerde::None,
                &default_column_option(),
                &None,
                false,
                false,
                &TokenStream::new(),
                &bonus_attributes([r#"serde(rename_all = "camelCase")"#, "ts(export)"]),
                &TokenStream::new(),
                false,
                true,
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
                        unique_key: None,
                    },
                    Column {
                        name: "payload".to_owned(),
                        col_type: ColumnType::Json,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "payload_binary".to_owned(),
                        col_type: ColumnType::JsonBinary,
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                ],
                relations: vec![],
                conjunct_relations: vec![],
                is_view: false,
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
                    &default_column_option(),
                    &None,
                    false,
                    false,
                    &TokenStream::new(),
                    &TokenStream::new(),
                    &TokenStream::new(),
                    false,
                    true,
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
                    &default_column_option(),
                    &Some("schema_name".to_owned()),
                    false,
                    false,
                    &TokenStream::new(),
                    &TokenStream::new(),
                    &TokenStream::new(),
                    false,
                    true,
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
    fn test_gen_import_active_enum() -> io::Result<()> {
        let entities = vec![
            Entity {
                table_name: "tea_pairing".to_owned(),
                columns: vec![
                    Column {
                        name: "id".to_owned(),
                        col_type: ColumnType::Integer,
                        auto_increment: true,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "first_tea".to_owned(),
                        col_type: ColumnType::Enum {
                            name: SeaRc::new(Alias::new("tea_enum")),
                            variants: vec![
                                SeaRc::new(Alias::new("everyday_tea")),
                                SeaRc::new(Alias::new("breakfast_tea")),
                            ],
                        },
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "second_tea".to_owned(),
                        col_type: ColumnType::Enum {
                            name: SeaRc::new(Alias::new("tea_enum")),
                            variants: vec![
                                SeaRc::new(Alias::new("everyday_tea")),
                                SeaRc::new(Alias::new("breakfast_tea")),
                            ],
                        },
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                ],
                relations: vec![],
                conjunct_relations: vec![],
                is_view: false,
                primary_keys: vec![PrimaryKey {
                    name: "id".to_owned(),
                }],
            },
            Entity {
                table_name: "tea_pairing_with_size".to_owned(),
                columns: vec![
                    Column {
                        name: "id".to_owned(),
                        col_type: ColumnType::Integer,
                        auto_increment: true,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "first_tea".to_owned(),
                        col_type: ColumnType::Enum {
                            name: SeaRc::new(Alias::new("tea_enum")),
                            variants: vec![
                                SeaRc::new(Alias::new("everyday_tea")),
                                SeaRc::new(Alias::new("breakfast_tea")),
                            ],
                        },
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "second_tea".to_owned(),
                        col_type: ColumnType::Enum {
                            name: SeaRc::new(Alias::new("tea_enum")),
                            variants: vec![
                                SeaRc::new(Alias::new("everyday_tea")),
                                SeaRc::new(Alias::new("breakfast_tea")),
                            ],
                        },
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                    Column {
                        name: "size".to_owned(),
                        col_type: ColumnType::Enum {
                            name: SeaRc::new(Alias::new("tea_size")),
                            variants: vec![
                                SeaRc::new(Alias::new("small")),
                                SeaRc::new(Alias::new("medium")),
                                SeaRc::new(Alias::new("huge")),
                            ],
                        },
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                        unique_key: None,
                    },
                ],
                relations: vec![],
                conjunct_relations: vec![],
                is_view: false,
                primary_keys: vec![PrimaryKey {
                    name: "id".to_owned(),
                }],
            },
        ];

        assert_eq!(
            quote!(
                use super::sea_orm_active_enums::TeaEnum;
            )
            .to_string(),
            EntityWriter::gen_import_active_enum(&entities[0]).to_string()
        );

        assert_eq!(
            quote!(
                use super::sea_orm_active_enums::TeaEnum;
                use super::sea_orm_active_enums::TeaSize;
            )
            .to_string(),
            EntityWriter::gen_import_active_enum(&entities[1]).to_string()
        );

        Ok(())
    }

    #[test]
    fn test_gen_dense_code_blocks() -> io::Result<()> {
        let entities = setup();
        const ENTITY_FILES: [&str; 13] = [
            include_str!("../../tests/dense/cake.rs"),
            include_str!("../../tests/dense/cake_filling.rs"),
            include_str!("../../tests/dense/cake_filling_price.rs"),
            include_str!("../../tests/dense/filling.rs"),
            include_str!("../../tests/dense/fruit.rs"),
            include_str!("../../tests/dense/vendor.rs"),
            include_str!("../../tests/dense/rust_keyword.rs"),
            include_str!("../../tests/dense/cake_with_float.rs"),
            include_str!("../../tests/dense/cake_with_double.rs"),
            include_str!("../../tests/dense/collection.rs"),
            include_str!("../../tests/dense/collection_float.rs"),
            include_str!("../../tests/dense/parent.rs"),
            include_str!("../../tests/dense/child.rs"),
        ];

        assert_eq!(entities.len(), ENTITY_FILES.len());

        for (i, entity) in entities.iter().enumerate() {
            assert_eq!(
                parse_from_file(ENTITY_FILES[i].as_bytes())?.to_string(),
                EntityWriter::gen_dense_code_blocks(
                    entity,
                    &crate::WithSerde::None,
                    &default_column_option(),
                    &None,
                    false,
                    false,
                    &TokenStream::new(),
                    &TokenStream::new(),
                    &TokenStream::new(),
                    false,
                    true,
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
