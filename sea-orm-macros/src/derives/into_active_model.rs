use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{Meta, PathArguments, PathSegment, punctuated::Punctuated, token::Comma};

use super::util::GetMeta;

enum Error {
    InputNotStruct,
    Syn(syn::Error),
}

pub(super) enum IntoActiveModelField {
    /// `IntoActiveValue::into_active_value(self.field).into()`
    Normal(syn::Ident),
    /// Option<T> with fallback: `Some(v) => Set(v).into(), None => Set(expr).into()`
    WithDefault { ident: syn::Ident, expr: syn::Expr },
}

impl IntoActiveModelField {
    pub(super) fn ident(&self) -> &syn::Ident {
        match self {
            IntoActiveModelField::Normal(ident) => ident,
            IntoActiveModelField::WithDefault { ident, .. } => ident,
        }
    }
}

pub(super) struct DeriveIntoActiveModel {
    pub ident: syn::Ident,
    pub active_model: Option<syn::Type>,
    pub fields: Vec<IntoActiveModelField>,
    pub sets: Vec<(syn::Ident, syn::Expr)>,
    /// require all fields specified, no `..default::Default()`
    pub exhaustive: bool,
}

impl DeriveIntoActiveModel {
    fn new(input: syn::DeriveInput) -> Result<Self, Error> {
        let fields = match input.data {
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
                ..
            }) => named,
            _ => return Err(Error::InputNotStruct),
        };

        let mut active_model = None;
        let mut sets = Vec::new();
        let mut exhaustive = false;

        for attr in input.attrs.iter() {
            if !attr.path().is_ident("sea_orm") {
                continue;
            }

            // Container attributes
            if let Ok(list) = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated) {
                for meta in list {
                    if let Some(s) = meta.get_as_kv("active_model") {
                        active_model = Some(syn::parse_str::<syn::Type>(&s).map_err(Error::Syn)?);
                    }
                    if meta.exists("exhaustive") {
                        exhaustive = true;
                    }
                    if let Meta::List(meta_list) = &meta {
                        if meta_list.path.is_ident("set") {
                            let nested = meta_list
                                .parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                                .map_err(Error::Syn)?;
                            for nested_meta in nested {
                                if let Some(val) = nested_meta.get_as_kv_with_ident() {
                                    let (ident, expr_str) = val;
                                    let expr = syn::parse_str::<syn::Expr>(&expr_str)
                                        .map_err(Error::Syn)?;
                                    sets.push((ident, expr));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Field attributes
        let field_idents = fields
            .iter()
            .filter_map(|field| {
                let mut default_expr: Option<syn::Expr> = None;

                for attr in field.attrs.iter() {
                    if !attr.path().is_ident("sea_orm") {
                        continue;
                    }

                    // Handle each type of argument
                    if let Ok(list) =
                        attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                    {
                        for meta in list.iter() {
                            // Skip if ignore or skip is present
                            if meta.exists("ignore") || meta.exists("skip") {
                                return None;
                            }

                            if let Some(expr_str) = meta.get_as_kv("default") {
                                if let Ok(expr) = syn::parse_str::<syn::Expr>(&expr_str) {
                                    default_expr = Some(expr);
                                }
                            }
                        }
                    }
                }

                let ident = field.ident.as_ref().unwrap().clone();

                if let Some(expr) = default_expr {
                    Some(IntoActiveModelField::WithDefault { ident, expr })
                } else {
                    Some(IntoActiveModelField::Normal(ident))
                }
            })
            .collect();

        Ok(Self {
            ident: input.ident,
            active_model,
            fields: field_idents,
            sets,
            exhaustive,
        })
    }

    fn expand(&self) -> syn::Result<TokenStream> {
        let expanded_impl_into_active_model = self.impl_into_active_model();

        Ok(expanded_impl_into_active_model)
    }

    pub(super) fn impl_into_active_model(&self) -> TokenStream {
        let Self {
            ident,
            active_model,
            fields,
            sets,
            exhaustive,
        } = self;

        let mut active_model_ident = active_model
            .clone()
            .unwrap_or_else(|| syn::parse_str::<syn::Type>("ActiveModel").unwrap());

        let type_alias_definition = if is_qualified_type(&active_model_ident) {
            let type_alias = format_ident!("ActiveModelFor{ident}");
            let type_def = quote!( type #type_alias = #active_model_ident; );
            active_model_ident = syn::Type::Path(syn::TypePath {
                qself: None,
                path: syn::Path {
                    leading_colon: None,
                    segments: [PathSegment {
                        ident: type_alias,
                        arguments: PathArguments::None,
                    }]
                    .into_iter()
                    .collect(),
                },
            });
            type_def
        } else {
            quote!()
        };

        let field_idents: Vec<_> = fields.iter().map(|f| f.ident()).collect();
        let expanded_fields = fields.iter().map(|field| match field {
            IntoActiveModelField::Normal(ident) => quote!(
                sea_orm::IntoActiveValue::<_>::into_active_value(self.#ident).into()
            ),
            IntoActiveModelField::WithDefault { ident, expr } => quote!({
                match self.#ident {
                    Some(v) => sea_orm::ActiveValue::Set(v).into(),
                    None => sea_orm::ActiveValue::Set(#expr).into(),
                }
            }),
        });

        let (set_idents, set_exprs): (Vec<_>, Vec<_>) = sets.iter().cloned().unzip();
        let expanded_sets = set_exprs.iter().map(|expr| {
            quote!(
                sea_orm::ActiveValue::Set(#expr)
            )
        });

        let rest = if *exhaustive {
            quote!()
        } else {
            quote!(..::std::default::Default::default())
        };

        quote!(
            #type_alias_definition

            #[automatically_derived]
            impl sea_orm::IntoActiveModel<#active_model_ident> for #ident {
                fn into_active_model(self) -> #active_model_ident {
                    #active_model_ident {
                        #( #field_idents: #expanded_fields, )*
                        #( #set_idents: #expanded_sets, )*
                        #rest
                    }
                }
            }
        )
    }
}

/// Method to derive the ActiveModel from the [ActiveModelTrait](sea_orm::ActiveModelTrait)
pub fn expand_into_active_model(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let ident_span = input.ident.span();

    match DeriveIntoActiveModel::new(input) {
        Ok(model) => model.expand(),
        Err(Error::InputNotStruct) => Ok(quote_spanned! {
            ident_span => compile_error!("you can only derive IntoActiveModel on structs");
        }),
        Err(Error::Syn(err)) => Err(err),
    }
}

fn is_qualified_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(syn::TypePath { qself: Some(_), .. }))
}
