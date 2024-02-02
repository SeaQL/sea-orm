use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use sea_query::DynIden;
use std::fmt::Write;

use crate::WithSerde;

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
    ) -> TokenStream {
        let enum_name = &self.enum_name.to_string();
        let enum_iden = format_ident!("{}", enum_name.to_upper_camel_case());
        let values: Vec<String> = self.values.iter().map(|v| v.to_string()).collect();
        let variants = values.iter().map(|v| v.trim()).map(|v| {
            if v.chars().next().map(char::is_numeric).unwrap_or(false) {
                format_ident!("_{}", v)
            } else {
                let variant_name = v.to_upper_camel_case();
                if variant_name.is_empty() {
                    println!("Warning: item '{}' in the enumeration '{}' cannot be converted into a valid Rust enum member name. It will be converted to its corresponding UTF-8 encoding. You can modify it later as needed.", v, enum_name);
                    let mut ss = String::new();
                    for c in v.chars() {
                        if c.len_utf8() > 1 {
                            write!(&mut ss, "{c}").unwrap();
                        } else {
                            write!(&mut ss, "U{:04X}", c as u32).unwrap();
                        }
                    }
                    format_ident!("{}", ss)
                } else {
                    format_ident!("{}", variant_name)
                }
            }
        });

        let serde_derive = with_serde.extra_derive();
        let copy_derive = if with_copy_enums {
            quote! { , Copy }
        } else {
            quote! {}
        };

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::writer::{bonus_attributes, bonus_derive};
    use pretty_assertions::assert_eq;
    use sea_query::{Alias, IntoIden};

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
                &bonus_attributes([r#"serde(rename_all = "camelCase")"#])
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
                &bonus_attributes([r#"serde(rename_all = "camelCase")"#, "ts(export)"])
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
                }
            )
            .to_string()
        )
    }
}
