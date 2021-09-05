extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Data, DeriveInput, Fields, Lit, Meta, parse_macro_input, punctuated::Punctuated, token::Comma};

use convert_case::{Case, Casing};

mod derives;

#[proc_macro_derive(DeriveEntity, attributes(table))]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, attrs, .. } = parse_macro_input!(input);

    match derives::expand_derive_entity(ident, attrs) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(DerivePrimaryKey)]
pub fn derive_primary_key(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    match derives::expand_derive_primary_key(ident, data) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(DeriveColumn)]
pub fn derive_column(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    match derives::expand_derive_column(&ident, &data) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(DeriveCustomColumn)]
pub fn derive_custom_column(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    match derives::expand_derive_custom_column(&ident, &data) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(DeriveModel)]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    match derives::expand_derive_model(ident, data) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(DeriveActiveModel)]
pub fn derive_active_model(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    match derives::expand_derive_active_model(ident, data) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(DeriveActiveModelBehavior)]
pub fn derive_active_model_behavior(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    match derives::expand_derive_active_model_behavior(ident, data) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(FromQueryResult)]
pub fn derive_from_query_result(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    match derives::expand_derive_from_query_result(ident, data) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[doc(hidden)]
#[proc_macro_attribute]
pub fn test(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::ItemFn);

    let ret = &input.sig.output;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;

    quote::quote! (
        #[test]
        #(#attrs)*
        fn #name() #ret {
            crate::block_on!(async { #body })
        }
    )
    .into()
}

#[proc_macro_derive(EntityModel, attributes(sea_orm))]
pub fn derive_entity_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if input.ident != "Model" {
        panic!("Struct name must be Model");
    }

    // if #[sea_orm(table_name = "foo")] specified, create Entity struct
    let table_name = input.attrs.iter().filter_map(|attr| {
        if attr.path.get_ident()? != "sea_orm" {
            return None;
        }

        let list = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated).ok()?;
        for meta in list.iter() {
            if let Meta::NameValue(nv) = meta {
                if nv.path.get_ident()? == "table_name" {
                    let table_name = &nv.lit;
                    return Some(quote! {
#[derive(Copy, Clone, Default, Debug, sea_orm::prelude::DeriveEntity)]
pub struct Entity;

impl sea_orm::prelude::EntityName for Entity {
    fn table_name(&self) -> &str {
        #table_name
    }
}
                    });
                }
            }
        }

        None
    }).next().unwrap_or_default();

    // generate Column enum and it's ColumnTrait impl
    let mut columns_enum: Punctuated<_, Comma> = Punctuated::new();
    let mut columns_trait: Punctuated<_, Comma> = Punctuated::new();
    let mut primary_keys: Punctuated<_, Comma> = Punctuated::new();
    if let Data::Struct(item_struct) = input.data {
        if let Fields::Named(fields) = item_struct.fields {
            for field in fields.named {
                if let Some(ident) = &field.ident {
                    let field_name = Ident::new(&ident.to_string().to_case(Case::Pascal), Span::call_site());
                    columns_enum.push(quote! { #field_name });

                    let mut nullable = false;
                    let mut sql_type = None;
                    // search for #[sea_orm(primary_key, type = "String", nullable)]
                    field.attrs.iter().for_each(|attr| {
                        if let Some(ident) = attr.path.get_ident() {
                            if ident != "sea_orm" {
                                return;
                            }
                        }
                        else {
                            return;
                        }

                        // single param
                        if let Ok(list) = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated) {
                            for meta in list.iter() {
                                match meta {
                                    Meta::NameValue(nv) => {
                                        if let Some(name) = nv.path.get_ident() {
                                            if name == "type" {
                                                if let Lit::Str(litstr) = &nv.lit {
                                                    let ty: TokenStream2 = syn::parse_str(&litstr.value()).unwrap();
                                                    sql_type = Some(ty);
                                                }
                                            }
                                        }
                                    },
                                    Meta::Path(p) => {
                                        if let Some(name) = p.get_ident() {
                                            if name == "primary_key" {
                                                primary_keys.push(quote! { #field_name });
                                            }
                                            else if name == "nullable" {
                                                nullable = true;
                                            }
                                        }
                                    },
                                    _ => {},
                                }
                            }
                        }
                    });
                    let field_type = sql_type.unwrap_or_else(|| {
                        let field_type = &field.ty;
                        let temp = quote! { #field_type }
                            .to_string()//Example: "Option < String >"
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
                            _ => panic!("unrecognized type {}", temp),
                        }
                    });

                    if nullable {
                        columns_trait.push(quote! { Self::#field_name => sea_orm::prelude::ColumnType::#field_type.def().null() });
                    }
                    else {
                        columns_trait.push(quote! { Self::#field_name => sea_orm::prelude::ColumnType::#field_type.def() });
                    }
                }
            }
        }
    }

    let primary_key = (!primary_keys.is_empty()).then(|| {
        let auto_increment = primary_keys.len() == 1;
        quote! {
#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    #primary_keys
}

impl PrimaryKeyTrait for PrimaryKey {
    fn auto_increment() -> bool {
        #auto_increment
    }
}
        }
    }).unwrap_or_default();

    return quote! {
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

#table_name

#primary_key
    }.into();
}
