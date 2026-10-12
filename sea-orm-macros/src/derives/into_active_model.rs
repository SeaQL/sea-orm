use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{Meta, PathArguments, PathSegment, punctuated::Punctuated, token::Comma};

enum Error {
    InputNotStruct,
    Syn(syn::Error),
}

/// An input field's mapping to an ActiveModel field and its optional fallback.
pub(super) struct IntoActiveModelField {
    pub source: syn::Ident,
    pub target: syn::Ident,
    pub default: Option<syn::Expr>,
}

/// Contains all the information extracted from the input struct and its attributes
/// needed to generate the `IntoActiveModel` trait implementation.
pub(super) struct DeriveIntoActiveModel {
    /// The identifier of the input struct
    pub ident: syn::Ident,
    /// Optional explicit ActiveModel type specified via `#[sea_orm(active_model = "Type")]`
    pub active_model: Option<syn::Type>,
    pub fields: Vec<IntoActiveModelField>,
    /// handles fields set by #[sea_orm(set(field = expr))]
    pub set_fields: Vec<(syn::Ident, syn::Expr)>,
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
        let mut set_fields = Vec::new();
        let mut exhaustive = false;

        for attr in input.attrs.iter() {
            if !attr.path().is_ident("sea_orm") {
                continue;
            }

            // Parse container attributes: #[sea_orm(...)]
            // Supports:
            // - active_model = "Type": explicitly specify the ActiveModel type
            // - exhaustive: require all ActiveModel fields to be explicitly set
            // - set(field = expr, ...): provide default values for fields not in the input struct
            let list = attr
                .parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                .map_err(Error::Syn)?;

            for meta in list {
                // Parse active_model attribute: #[sea_orm(active_model = "MyActiveModel")]
                if meta.path().is_ident("active_model") {
                    active_model = Some(to_lit_str(&meta)?.parse().map_err(Error::Syn)?);
                }
                // Parse exhaustive flag: #[sea_orm(exhaustive)]
                // When set, prevents using Default::default() for unspecified fields
                if meta.path().is_ident("exhaustive") {
                    meta.require_path_only().map_err(Error::Syn)?;
                    exhaustive = true;
                }
                // Parse set attribute: #[sea_orm(set(field1 = expr1, field2 = expr2, ...))]
                // Collects field assignments to be included in the generated ActiveModel
                if meta.path().is_ident("set") {
                    let nested = meta
                        .require_list()
                        .map_err(Error::Syn)?
                        .parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                        .map_err(Error::Syn)?;
                    for nested_meta in nested {
                        let ident = nested_meta
                            .path()
                            .require_ident()
                            .map_err(Error::Syn)?
                            .clone();
                        let expr = to_lit_str(&nested_meta)?.parse().map_err(Error::Syn)?;

                        set_fields.push((ident, expr));
                    }
                }
            }
        }

        // Field attributes
        let mut field_idents = Vec::new();
        for field in fields.iter() {
            if let Some(f) = parse_field(field)? {
                field_idents.push(f);
            }
        }

        Ok(Self {
            ident: input.ident,
            active_model,
            fields: field_idents,
            set_fields,
            exhaustive,
        })
    }

    fn expand(&self) -> syn::Result<TokenStream> {
        let expanded_impl_into_active_model = self.impl_into_active_model();

        Ok(expanded_impl_into_active_model)
    }

    /// Generates the implementation of `IntoActiveModel` trait for the input struct
    pub(super) fn impl_into_active_model(&self) -> TokenStream {
        let Self {
            ident,
            active_model,
            fields,
            set_fields,
            exhaustive,
        } = self;

        let mut active_model_ident = active_model
            .clone()
            .unwrap_or_else(|| syn::parse_str::<syn::Type>("ActiveModel").unwrap());

        // Create a type alias for qualified types
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

        // Generate field conversion code based on field type
        let expanded_fields = fields.iter().map(
            |IntoActiveModelField {
                 source,
                 target,
                 default,
             }| {
                let value = match default {
                    None => quote!(
                        sea_orm::IntoActiveValue::<_>::into_active_value(self.#source).into()
                    ),
                    Some(expr) => quote!({
                        match self.#source.into() {
                            Some(v) => sea_orm::ActiveValue::Set(v).into(),
                            None => sea_orm::ActiveValue::Set(#expr).into(),
                        }
                    }),
                };

                quote!(#target: #value)
            },
        );

        // Add custom field assignments from #[sea_orm(set(field = expr))]
        let expanded_sets = set_fields.iter().map(|(target, expr)| {
            quote!(
                #target: sea_orm::ActiveValue::Set(#expr)
            )
        });

        // Add defaults(Unset) unless exhaustive mode is enabled
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
                        #( #expanded_fields, )*
                        #( #expanded_sets, )*
                        #rest
                    }
                }
            }
        )
    }
}

fn parse_field(field: &syn::Field) -> Result<Option<IntoActiveModelField>, Error> {
    let ident = field.ident.as_ref().unwrap().clone();

    let mut target = ident.clone();

    let mut default_expr: Option<syn::Expr> = None;

    let mut ignored = false;

    for attr in field
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("sea_orm"))
    {
        // Parse the attribute arguments: #[sea_orm(...)]
        let list = attr
            .parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
            .map_err(Error::Syn)?;

        for meta in &list {
            // Check for ignore/skip: #[sea_orm(ignore)] or #[sea_orm(skip)]
            if meta.path().is_ident("ignore") || meta.path().is_ident("skip") {
                meta.require_path_only().map_err(Error::Syn)?;
                ignored = true;
            }

            if meta.path().is_ident("rename") {
                target = to_lit_str(meta)?.parse().map_err(Error::Syn)?;
            }

            if meta.path().is_ident("default") {
                if default_expr.is_some() {
                    return Err(Error::Syn(syn::Error::new_spanned(
                        meta,
                        "duplicate `default` attribute",
                    )));
                }
                default_expr = Some(match meta {
                    Meta::Path(_) => syn::parse_quote!(::core::default::Default::default()),
                    _ => to_lit_str(meta)?.parse().map_err(Error::Syn)?,
                });
            }
        }
    }

    if ignored {
        return Ok(None);
    }

    Ok(Some(IntoActiveModelField {
        source: ident,
        target,
        default: default_expr,
    }))
}

fn to_lit_str(meta: &Meta) -> Result<&syn::LitStr, Error> {
    let value = &meta.require_name_value().map_err(Error::Syn)?.value;

    let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(value),
        ..
    }) = value
    else {
        return Err(Error::Syn(syn::Error::new_spanned(
            value,
            "expected a string literal",
        )));
    };

    Ok(value)
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
