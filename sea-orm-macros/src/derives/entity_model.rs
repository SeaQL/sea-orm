
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Attribute, Data, Fields, Lit, Meta, parse::Error, punctuated::Punctuated, spanned::Spanned, token::Comma};

use convert_case::{Case, Casing};

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
                        }
                        else if ident == "schema_name" {
                            let name = &nv.lit;
                            schema_name = quote! { Some(#name) };
                        }
                    }
                }
            }
        }
    });
    let entity_def = table_name.map(|table_name| quote! {
            #[derive(Copy, Clone, Default, Debug, sea_orm::prelude::DeriveEntity)]
            pub struct Entity;
            
            impl sea_orm::prelude::EntityName for Entity {
                fn schema_name(&self) -> &str {
                    #schema_name
                }

                fn table_name(&self) -> &str {
                    #table_name
                }
            }
        }).unwrap_or_default();

    // generate Column enum and it's ColumnTrait impl
    let mut columns_enum: Punctuated<_, Comma> = Punctuated::new();
    let mut columns_trait: Punctuated<_, Comma> = Punctuated::new();
    let mut primary_keys: Punctuated<_, Comma> = Punctuated::new();
    let mut primary_key_types: Punctuated<_, Comma> = Punctuated::new();
    let mut auto_increment = true;
    if let Data::Struct(item_struct) = data {
        if let Fields::Named(fields) = item_struct.fields {
            for field in fields.named {
                if let Some(ident) = &field.ident {
                    let field_name = Ident::new(&ident.to_string().to_case(Case::Pascal), Span::call_site());
                    columns_enum.push(quote! { #field_name });

                    let mut nullable = false;
                    let mut default_value = None;
                    let mut default_expr = None;
                    let mut indexed = false;
                    let mut unique = false;
                    let mut sql_type = None;
                    // search for #[sea_orm(primary_key, auto_increment = false, column_type = "String(Some(255))", default_value = "new user", default_expr = "gen_random_uuid()", nullable, indexed, unique)]
                    for attr in field.attrs.iter() {
                        if let Some(ident) = attr.path.get_ident() {
                            if ident != "sea_orm" {
                                continue;
                            }
                        }
                        else {
                            continue;
                        }

                        // single param
                        if let Ok(list) = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated) {
                            for meta in list.iter() {
                                match meta {
                                    Meta::NameValue(nv) => {
                                        if let Some(name) = nv.path.get_ident() {
                                            if name == "column_type" {
                                                if let Lit::Str(litstr) = &nv.lit {
                                                    let ty: TokenStream = syn::parse_str(&litstr.value())?;
                                                    sql_type = Some(ty);
                                                }
                                                else {
                                                    return Err(Error::new(field.span(), format!("Invalid column_type {:?}", nv.lit)));
                                                }
                                            }
                                            else if name == "auto_increment" {
                                                if let Lit::Str(litstr) = &nv.lit {
                                                    auto_increment = match litstr.value().as_str() {
                                                        "true" => true,
                                                        "false" => false,
                                                        _ => return Err(Error::new(field.span(), format!("Invalid auto_increment = {}", litstr.value()))),
                                                    };
                                                }
                                                else {
                                                    return Err(Error::new(field.span(), format!("Invalid auto_increment = {:?}", nv.lit)));
                                                }
                                            }
                                            else if name == "default_value" {
                                                default_value = Some(nv.lit.to_owned());
                                            }
                                            else if name == "default_expr" {
                                                default_expr = Some(nv.lit.to_owned());
                                            }
                                        }
                                    },
                                    Meta::Path(p) => {
                                        if let Some(name) = p.get_ident() {
                                            if name == "primary_key" {
                                                primary_keys.push(quote! { #field_name });
                                                primary_key_types.push(field.ty.clone());
                                            }
                                            else if name == "nullable" {
                                                nullable = true;
                                            }
                                            else if name == "indexed" {
                                                indexed = true;
                                            }
                                            else if name == "unique" {
                                                unique = true;
                                            }
                                        }
                                    },
                                    _ => {},
                                }
                            }
                        }
                    }

                    let field_type = match sql_type {
                        Some(t) => t,
                        None => {
                            let field_type = &field.ty;
                            let temp = quote! { #field_type }
                                .to_string()//E.g.: "Option < String >"
                                .replace(" ", "");
                            let temp = if temp.starts_with("Option<") {
                                nullable = true;
                                &temp[7..(temp.len() - 1)]
                            }
                            else {
                                temp.as_str()
                            };
                            match temp {
                                "char" => quote! { Char(None) },
                                "String" | "&str" => quote! { String(None) },
                                "u8" | "i8" => quote! { TinyInteger },
                                "u16" | "i16" => quote! { SmallInteger },
                                "u32" | "u64" | "i32" | "i64" => quote! { Integer },
                                "u128" | "i128" => quote! { BigInteger },
                                "f32" => quote! { Float },
                                "f64" => quote! { Double },
                                "bool" => quote! { Boolean },
                                "NaiveDate" => quote! { Date },
                                "NaiveTime" => quote! { Time },
                                "NaiveDateTime" => quote! { DateTime },
                                "Uuid" => quote! { Uuid },
                                "Decimal" => quote! { BigInteger },
                                _ => return Err(Error::new(field.span(), format!("unrecognized type {}", temp))),
                            }
                        }
                    };

                    let mut match_row = quote! { Self::#field_name => sea_orm::prelude::ColumnType::#field_type.def() };
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

    let primary_key = (!primary_keys.is_empty()).then(|| {
        let auto_increment = auto_increment && primary_keys.len() == 1;
        let primary_key_types = if primary_key_types.len() == 1 {
            let first = primary_key_types.first();
            quote! { #first }
        }
        else {
            quote! { (#primary_key_types) }
        };
        quote! {
#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    #primary_keys
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = #primary_key_types;

    fn auto_increment() -> bool {
        #auto_increment
    }
}
        }
    }).unwrap_or_default();

    return Ok(quote! {
#[derive(Copy, Clone, Debug, sea_orm::prelude::EnumIter, sea_orm::prelude::DeriveColumn)]
pub enum Column {
    #columns_enum
}

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
