use crate::util::{escape_rust_keyword, trim_starting_raw_identifier};
use heck::CamelCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{
    parse::Error, punctuated::Punctuated, spanned::Spanned, token::Comma, Attribute, Data, Fields,
    Lit, Meta,
};

/// Method to derive an Model
pub fn expand_derive_entity_model(data: Data, attrs: Vec<Attribute>) -> syn::Result<TokenStream> {
    // if #[sea_orm(table_name = "foo", schema_name = "bar")] specified, create Entity struct
    let mut table_name = None;
    let mut schema_name = quote! { None };
    attrs.iter().for_each(|attr| {
        if attr.path.get_ident().map(|i| i == "sea_orm") != Some(true) {
            return;
        }

        if let Ok(list) = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated) {
            for meta in list.iter() {
                if let Meta::NameValue(nv) = meta {
                    if let Some(ident) = nv.path.get_ident() {
                        if ident == "table_name" {
                            table_name = Some(nv.lit.clone());
                        } else if ident == "schema_name" {
                            let name = &nv.lit;
                            schema_name = quote! { Some(#name) };
                        }
                    }
                }
            }
        }
    });
    let entity_def = table_name
        .as_ref()
        .map(|table_name| {
            quote! {
                #[derive(Copy, Clone, Default, Debug, sea_orm::prelude::DeriveEntity)]
                pub struct Entity;

                #[automatically_derived]
                impl sea_orm::prelude::EntityName for Entity {
                    fn schema_name(&self) -> Option<&str> {
                        #schema_name
                    }

                    fn table_name(&self) -> &str {
                        #table_name
                    }
                }
            }
        })
        .unwrap_or_default();

    // generate Column enum and it's ColumnTrait impl
    let mut columns_enum: Punctuated<_, Comma> = Punctuated::new();
    let mut columns_trait: Punctuated<_, Comma> = Punctuated::new();
    let mut primary_keys: Punctuated<_, Comma> = Punctuated::new();
    let mut primary_key_types: Punctuated<_, Comma> = Punctuated::new();
    let mut auto_increment = true;

    #[cfg(feature = "with-table-iden")]
    if let Some(table_name) = table_name {
        let table_field_name = Ident::new("Table", Span::call_site());
        columns_enum.push(quote! {
            #[sea_orm(table_name=#table_name)]
            #[strum(disabled)]
            #table_field_name
        });
        columns_trait
            .push(quote! { Self::#table_field_name => panic!("Table cannot be used as a column") });
    }
    if let Data::Struct(item_struct) = data {
        if let Fields::Named(fields) = item_struct.fields {
            for field in fields.named {
                if let Some(ident) = &field.ident {
                    let mut field_name = Ident::new(
                        &trim_starting_raw_identifier(&ident).to_camel_case(),
                        Span::call_site(),
                    );

                    let mut nullable = false;
                    let mut default_value = None;
                    let mut default_expr = None;
                    let mut indexed = false;
                    let mut ignore = false;
                    let mut unique = false;
                    let mut sql_type = None;
                    let mut column_name = None;
                    let mut enum_name = None;
                    let mut is_primary_key = false;
                    // search for #[sea_orm(primary_key, auto_increment = false, column_type = "String(Some(255))", default_value = "new user", default_expr = "gen_random_uuid()", column_name = "name", enum_name = "Name", nullable, indexed, unique)]
                    for attr in field.attrs.iter() {
                        if let Some(ident) = attr.path.get_ident() {
                            if ident != "sea_orm" {
                                continue;
                            }
                        } else {
                            continue;
                        }

                        // single param
                        if let Ok(list) =
                            attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                        {
                            for meta in list.iter() {
                                match meta {
                                    Meta::NameValue(nv) => {
                                        if let Some(name) = nv.path.get_ident() {
                                            if name == "column_type" {
                                                if let Lit::Str(litstr) = &nv.lit {
                                                    let ty: TokenStream =
                                                        syn::parse_str(&litstr.value())?;
                                                    sql_type = Some(ty);
                                                } else {
                                                    return Err(Error::new(
                                                        field.span(),
                                                        format!("Invalid column_type {:?}", nv.lit),
                                                    ));
                                                }
                                            } else if name == "auto_increment" {
                                                if let Lit::Bool(litbool) = &nv.lit {
                                                    auto_increment = litbool.value();
                                                } else {
                                                    return Err(Error::new(
                                                        field.span(),
                                                        format!(
                                                            "Invalid auto_increment = {:?}",
                                                            nv.lit
                                                        ),
                                                    ));
                                                }
                                            } else if name == "default_value" {
                                                default_value = Some(nv.lit.to_owned());
                                            } else if name == "default_expr" {
                                                default_expr = Some(nv.lit.to_owned());
                                            } else if name == "column_name" {
                                                if let Lit::Str(litstr) = &nv.lit {
                                                    column_name = Some(litstr.value());
                                                } else {
                                                    return Err(Error::new(
                                                        field.span(),
                                                        format!("Invalid column_name {:?}", nv.lit),
                                                    ));
                                                }
                                            } else if name == "enum_name" {
                                                if let Lit::Str(litstr) = &nv.lit {
                                                    let ty: Ident =
                                                        syn::parse_str(&litstr.value())?;
                                                    enum_name = Some(ty);
                                                } else {
                                                    return Err(Error::new(
                                                        field.span(),
                                                        format!("Invalid enum_name {:?}", nv.lit),
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                    Meta::Path(p) => {
                                        if let Some(name) = p.get_ident() {
                                            if name == "ignore" {
                                                ignore = true;
                                                break;
                                            } else if name == "primary_key" {
                                                is_primary_key = true;
                                                primary_key_types.push(field.ty.clone());
                                            } else if name == "nullable" {
                                                nullable = true;
                                            } else if name == "indexed" {
                                                indexed = true;
                                            } else if name == "unique" {
                                                unique = true;
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    if let Some(enum_name) = enum_name {
                        field_name = enum_name;
                    }

                    field_name = Ident::new(&escape_rust_keyword(field_name), Span::call_site());

                    if ignore {
                        continue;
                    } else {
                        let variant_attrs = match &column_name {
                            Some(column_name) => quote! {
                                #[sea_orm(column_name = #column_name)]
                            },
                            None => quote! {},
                        };
                        columns_enum.push(quote! {
                            #variant_attrs
                            #field_name
                        });
                    }

                    if is_primary_key {
                        primary_keys.push(quote! { #field_name });
                    }

                    let col_type = match sql_type {
                        Some(t) => quote! { sea_orm::prelude::ColumnType::#t.def() },
                        None => {
                            let field_type = &field.ty;
                            let temp = quote! { #field_type }
                                .to_string() //E.g.: "Option < String >"
                                .replace(" ", "");
                            let temp = if temp.starts_with("Option<") {
                                nullable = true;
                                &temp[7..(temp.len() - 1)]
                            } else {
                                temp.as_str()
                            };
                            let col_type = match temp {
                                "char" => quote! { Char(None) },
                                "String" | "&str" => quote! { String(None) },
                                "u8" | "i8" => quote! { TinyInteger },
                                "u16" | "i16" => quote! { SmallInteger },
                                "u32" | "i32" => quote! { Integer },
                                "u64" | "i64" => quote! { BigInteger },
                                "f32" => quote! { Float },
                                "f64" => quote! { Double },
                                "bool" => quote! { Boolean },
                                "Date" | "NaiveDate" => quote! { Date },
                                "Time" | "NaiveTime" => quote! { Time },
                                "DateTime" | "NaiveDateTime" => {
                                    quote! { DateTime }
                                }
                                "DateTimeWithTimeZone" => {
                                    quote! { TimestampWithTimeZone }
                                }
                                "Uuid" => quote! { Uuid },
                                "Json" => quote! { Json },
                                "Decimal" => quote! { Decimal(None) },
                                "Vec<u8>" => quote! { Binary },
                                _ => {
                                    // Assumed it's ActiveEnum if none of the above type matches
                                    quote! {}
                                }
                            };
                            if col_type.is_empty() {
                                let field_span = field.span();
                                let ty = format_ident!("{}", temp);
                                let def = quote_spanned! { field_span => {
                                    std::convert::Into::<sea_orm::ColumnType>::into(
                                        <#ty as sea_orm::sea_query::ValueType>::column_type()
                                    )
                                    .def()
                                }};
                                quote! { #def }
                            } else {
                                quote! { sea_orm::prelude::ColumnType::#col_type.def() }
                            }
                        }
                    };

                    let mut match_row = quote! { Self::#field_name => #col_type };
                    if nullable {
                        match_row = quote! { #match_row.nullable() };
                    }
                    if indexed {
                        match_row = quote! { #match_row.indexed() };
                    }
                    if unique {
                        match_row = quote! { #match_row.unique() };
                    }
                    if let Some(default_value) = default_value {
                        match_row = quote! { #match_row.default_value(#default_value) };
                    }
                    if let Some(default_expr) = default_expr {
                        match_row = quote! { #match_row.default_expr(#default_expr) };
                    }
                    columns_trait.push(match_row);
                }
            }
        }
    }

    let primary_key = (!primary_keys.is_empty())
        .then(|| {
            let auto_increment = auto_increment && primary_keys.len() == 1;
            let primary_key_types = if primary_key_types.len() == 1 {
                let first = primary_key_types.first();
                quote! { #first }
            } else {
                quote! { (#primary_key_types) }
            };
            quote! {
            #[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
            pub enum PrimaryKey {
                #primary_keys
            }

            #[automatically_derived]
            impl PrimaryKeyTrait for PrimaryKey {
                type ValueType = #primary_key_types;

                fn auto_increment() -> bool {
                    #auto_increment
                }
            }
                    }
        })
        .unwrap_or_default();

    Ok(quote! {
        #[derive(Copy, Clone, Debug, sea_orm::prelude::EnumIter, sea_orm::prelude::DeriveColumn)]
        pub enum Column {
            #columns_enum
        }

        #[automatically_derived]
        impl sea_orm::prelude::ColumnTrait for Column {
            type EntityName = Entity;

            fn def(&self) -> sea_orm::prelude::ColumnDef {
                match self {
                    #columns_trait
                }
            }
        }

        #entity_def

        #primary_key
    })
}
