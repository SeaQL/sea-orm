use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use sea_query::DynIden;

use crate::{EntityFormat, WithSerde};

#[derive(Clone, Debug)]
pub struct ActiveEnum {
    pub(crate) enum_name: DynIden,
    pub(crate) values: Vec<DynIden>,
}

impl ActiveEnum {
    pub fn impl_active_enum(
        &self,
        with_serde: &WithSerde,
        with_copy_enums: bool,
        extra_derives: &TokenStream,
        extra_attributes: &TokenStream,
        entity_format: EntityFormat,
    ) -> TokenStream {
        let enum_name = &self.enum_name.to_string();
        let enum_iden = format_ident!("{}", enum_name.to_upper_camel_case());
        let values: Vec<String> = self.values.iter().map(|v| v.to_string()).collect();

        let variants = values.iter().map(|v| v.trim()).map(|v| {
            if v.is_empty() {
                println!("Warning: item in the enumeration '{enum_name}' is an empty string, it will be converted to `__EmptyString`. You can modify it later as needed.");
                return format_ident!("__EmptyString");
            }

            let is_leading_digit = v.chars().next().is_some_and(|c| c.is_ascii_digit());
            let is_valid_char = |c: char| c.is_ascii_alphanumeric() || c == '_';
            let is_passthrough = |c: char| matches!(c, '-' | ' ');
            let needs_utf8_escape = |c: char| !is_valid_char(c) && !is_passthrough(c);

            if v.chars().any(needs_utf8_escape) {
                println!("Warning: item '{v}' in the enumeration '{enum_name}' cannot be converted into a valid Rust enum member name. It will be converted to its corresponding UTF-8 encoding. You can modify it later as needed.");

                let mut buf = String::new();

                if is_leading_digit {
                    buf.push('_');
                }

                for c in v.chars() {
                    if is_passthrough(c) {
                        continue;
                    } else if is_valid_char(c) || c.len_utf8() > 1 {
                        buf.push(c);
                    } else {
                        buf.push_str(&format!("U{:04X}", c as u32));
                    }
                }

                return format_ident!("{buf}");
            }

            if is_leading_digit {
                let sanitized = v.chars().filter(|&c| is_valid_char(c)).collect::<String>();
                format_ident!("_{sanitized}")
            } else {
                format_ident!("{}", v.to_upper_camel_case())
            }
        });

        let serde_derive = with_serde.extra_derive();
        let copy_derive = if with_copy_enums {
            quote! { , Copy }
        } else {
            quote! {}
        };

        if entity_format == EntityFormat::Frontend {
            quote! {
                #[derive(Debug, Clone, PartialEq, Eq #copy_derive #serde_derive #extra_derives)]
                #extra_attributes
                pub enum #enum_iden {
                    #(
                        #variants,
                    )*
                }
            }
        } else {
            quote! {
                #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum #copy_derive #serde_derive #extra_derives)]
                #[sea_orm(rs_type = "String", db_type = "Enum", enum_name = #enum_name)]
                #extra_attributes
                pub enum #enum_iden {
                    #(
                        #[sea_orm(string_value = #values)]
                        #variants,
                    )*
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::writer::{bonus_attributes, bonus_derive};
    use pretty_assertions::assert_eq;
    use sea_query::{Alias, IntoIden};

    #[test]
    fn test_enum_variant_starts_with_empty_string() {
        assert_eq!(
            ActiveEnum {
                enum_name: Alias::new("media_type").into_iden(),
                values: vec![""]
                    .into_iter()
                    .map(|variant| Alias::new(variant).into_iden())
                    .collect(),
            }
            .impl_active_enum(
                &WithSerde::None,
                true,
                &TokenStream::new(),
                &TokenStream::new(),
                EntityFormat::Compact,
            )
            .to_string(),
            quote!(
                #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Copy)]
                #[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "media_type")]
                pub enum MediaType {
                    #[sea_orm(string_value = "")]
                    __EmptyString,
                }
            )
            .to_string()
        )
    }

    #[test]
    fn test_enum_variant_starts_with_number() {
        assert_eq!(
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
                    "2-D"
                ]
                .into_iter()
                .map(|variant| Alias::new(variant).into_iden())
                .collect(),
            }
            .impl_active_enum(
                &WithSerde::None,
                true,
                &TokenStream::new(),
                &TokenStream::new(),
                EntityFormat::Compact,
            )
            .to_string(),
            quote!(
                #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Copy)]
                #[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "media_type")]
                pub enum MediaType {
                    #[sea_orm(string_value = "UNKNOWN")]
                    Unknown,
                    #[sea_orm(string_value = "BITMAP")]
                    Bitmap,
                    #[sea_orm(string_value = "DRAWING")]
                    Drawing,
                    #[sea_orm(string_value = "AUDIO")]
                    Audio,
                    #[sea_orm(string_value = "VIDEO")]
                    Video,
                    #[sea_orm(string_value = "MULTIMEDIA")]
                    Multimedia,
                    #[sea_orm(string_value = "OFFICE")]
                    Office,
                    #[sea_orm(string_value = "TEXT")]
                    Text,
                    #[sea_orm(string_value = "EXECUTABLE")]
                    Executable,
                    #[sea_orm(string_value = "ARCHIVE")]
                    Archive,
                    #[sea_orm(string_value = "3D")]
                    _3D,
                    #[sea_orm(string_value = "2-D")]
                    _2D,
                }
            )
            .to_string()
        )
    }

    #[test]
    fn test_enum_extra_derives() {
        assert_eq!(
            ActiveEnum {
                enum_name: Alias::new("media_type").into_iden(),
                values: vec!["UNKNOWN", "BITMAP",]
                    .into_iter()
                    .map(|variant| Alias::new(variant).into_iden())
                    .collect(),
            }
            .impl_active_enum(
                &WithSerde::None,
                true,
                &bonus_derive(["specta::Type", "ts_rs::TS"]),
                &TokenStream::new(),
                EntityFormat::Compact,
            )
            .to_string(),
            build_generated_enum(),
        );

        #[rustfmt::skip]
        fn build_generated_enum() -> String {
            quote!(
                #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Copy, specta :: Type, ts_rs :: TS)]
                #[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "media_type")]
                pub enum MediaType {
                    #[sea_orm(string_value = "UNKNOWN")]
                    Unknown,
                    #[sea_orm(string_value = "BITMAP")]
                    Bitmap,
                }
            )
            .to_string()
        }
    }

    #[test]
    fn test_enum_extra_attributes() {
        assert_eq!(
            ActiveEnum {
                enum_name: Alias::new("coinflip_result_type").into_iden(),
                values: vec!["HEADS", "TAILS"]
                    .into_iter()
                    .map(|variant| Alias::new(variant).into_iden())
                    .collect(),
            }
            .impl_active_enum(
                &WithSerde::None,
                true,
                &TokenStream::new(),
                &bonus_attributes([r#"serde(rename_all = "camelCase")"#]),
                EntityFormat::Compact,
            )
            .to_string(),
            quote!(
                #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Copy)]
                #[sea_orm(
                    rs_type = "String",
                    db_type = "Enum",
                    enum_name = "coinflip_result_type"
                )]
                #[serde(rename_all = "camelCase")]
                pub enum CoinflipResultType {
                    #[sea_orm(string_value = "HEADS")]
                    Heads,
                    #[sea_orm(string_value = "TAILS")]
                    Tails,
                }
            )
            .to_string()
        );
        assert_eq!(
            ActiveEnum {
                enum_name: Alias::new("coinflip_result_type").into_iden(),
                values: vec!["HEADS", "TAILS"]
                    .into_iter()
                    .map(|variant| Alias::new(variant).into_iden())
                    .collect(),
            }
            .impl_active_enum(
                &WithSerde::None,
                true,
                &TokenStream::new(),
                &bonus_attributes([r#"serde(rename_all = "camelCase")"#, "ts(export)"]),
                EntityFormat::Compact,
            )
            .to_string(),
            quote!(
                #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Copy)]
                #[sea_orm(
                    rs_type = "String",
                    db_type = "Enum",
                    enum_name = "coinflip_result_type"
                )]
                #[serde(rename_all = "camelCase")]
                #[ts(export)]
                pub enum CoinflipResultType {
                    #[sea_orm(string_value = "HEADS")]
                    Heads,
                    #[sea_orm(string_value = "TAILS")]
                    Tails,
                }
            )
            .to_string()
        )
    }

    #[test]
    fn test_enum_variant_utf8_encode() {
        assert_eq!(
            ActiveEnum {
                enum_name: Alias::new("ty").into_iden(),
                values: vec![
                    "Question",
                    "QuestionsAdditional",
                    "Answer",
                    "Other",
                    "/",
                    "//",
                    "A-B-C",
                    "你好",
                    "0/",
                    "0//",
                    "0A-B-C",
                    "0你好",
                ]
                .into_iter()
                .map(|variant| Alias::new(variant).into_iden())
                .collect(),
            }
            .impl_active_enum(
                &WithSerde::None,
                true,
                &TokenStream::new(),
                &TokenStream::new(),
                EntityFormat::Compact,
            )
            .to_string(),
            quote!(
                #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Copy)]
                #[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "ty")]
                pub enum Ty {
                    #[sea_orm(string_value = "Question")]
                    Question,
                    #[sea_orm(string_value = "QuestionsAdditional")]
                    QuestionsAdditional,
                    #[sea_orm(string_value = "Answer")]
                    Answer,
                    #[sea_orm(string_value = "Other")]
                    Other,
                    #[sea_orm(string_value = "/")]
                    U002F,
                    #[sea_orm(string_value = "//")]
                    U002FU002F,
                    #[sea_orm(string_value = "A-B-C")]
                    ABC,
                    #[sea_orm(string_value = "你好")]
                    你好,
                    #[sea_orm(string_value = "0/")]
                    _0U002F,
                    #[sea_orm(string_value = "0//")]
                    _0U002FU002F,
                    #[sea_orm(string_value = "0A-B-C")]
                    _0ABC,
                    #[sea_orm(string_value = "0你好")]
                    _0你好,
                }
            )
            .to_string()
        )
    }
}
