use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{Meta, PathArguments, PathSegment, punctuated::Punctuated, token::Comma};

use super::util::GetMeta;

enum Error {
    InputNotStruct,
    Syn(syn::Error),
}

/// Matches all potential ways to convert struct fields into ActiveModel ones
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

/// Contains all the information extracted from the input struct and its attributes
/// needed to generate the `IntoActiveModel` trait implementation.
pub(super) struct DeriveIntoActiveModel {
    /// The identifier of the input struct
    pub ident: syn::Ident,
    /// Optional explicit ActiveModel type specified via `#[sea_orm(active_model = "Type")]`
    pub active_model: Option<syn::Type>,
    /// handles provided struct fields
    pub fields: Vec<IntoActiveModelField>,
    /// handles fields set by #[sea_orm(set(field = expr))]
    pub set_fields: Vec<(syn::Ident, syn::Expr)>,
    /// require all fields specified, no `..default::Default()`
    pub exhaustive: bool,
}

impl DeriveIntoActiveModel {
    /// This function finds attributes relevant for this macros:
    /// Container attributes (#[sea_orm(...)]) on the struct for:
    ///   - active_model: explicit ActiveModel type
    ///   - exhaustive: require all fields to be set
    ///   - set/fill(...): provided values for ommited fields
    ///
    /// Field attributes (#[sea_orm(...)]) with:
    ///   - ignore/skip: exclude from conversion
    ///   - default: fallback value for Option<T> fields
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
            if let Ok(list) = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated) {
                for meta in list {
                    // Parse active_model attribute: #[sea_orm(active_model = "MyActiveModel")]
                    if let Some(s) = meta.get_as_kv("active_model") {
                        active_model = Some(syn::parse_str::<syn::Type>(&s).map_err(Error::Syn)?);
                    }
                    // Parse exhaustive flag: #[sea_orm(exhaustive)]
                    // When set, prevents using Default::default() for unspecified fields
                    if meta.exists("exhaustive") {
                        exhaustive = true;
                    }
                    // Parse set/fill attribute: #[sea_orm(set(field1 = expr1, field2 = expr2, ...))]
                    // Collects field assignments to be included in the generated ActiveModel
                    if let Meta::List(meta_list) = &meta {
                        if meta_list.path.is_ident("set") || meta_list.path.is_ident("fill") {
                            let nested = meta_list
                                .parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                                .map_err(Error::Syn)?;
                            for nested_meta in nested {
                                if let Some(val) = nested_meta.get_as_kv_with_ident() {
                                    let (ident, expr_str) = val;
                                    let expr = syn::parse_str::<syn::Expr>(&expr_str)
                                        .map_err(Error::Syn)?;
                                    set_fields.push((ident, expr));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Field attributes
        let mut field_idents: Vec<IntoActiveModelField> = Vec::new();
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

        let field_idents: Vec<_> = fields.iter().map(|f| f.ident()).collect();

        // Generate field conversion code based on field type
        let expanded_fields = fields.iter().map(|field| match field {
            IntoActiveModelField::Normal(ident) => quote!(
                sea_orm::IntoActiveValue::<_>::into_active_value(self.#ident).into()
            ),
            IntoActiveModelField::WithDefault { ident, expr } => quote!({
                match self.#ident.into() {
                    Some(v) => sea_orm::ActiveValue::Set(v).into(),
                    None => sea_orm::ActiveValue::Set(#expr).into(),
                }
            }),
        });

        // Add custom field assignments from #[sea_orm(set(field = expr))]
        let (set_idents, set_exprs): (Vec<_>, Vec<_>) = set_fields.iter().cloned().unzip();
        let expanded_sets = set_exprs.iter().map(|expr| {
            quote!(
                sea_orm::ActiveValue::Set(#expr)
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
                        #( #field_idents: #expanded_fields, )*
                        #( #set_idents: #expanded_sets, )*
                        #rest
                    }
                }
            }
        )
    }
}

/// Parse field-level attributes on each struct field
/// Supports:
/// - ignore or skip: exclude the field from conversion
/// - default = "expr": provide a fallback value for Option<T> fields (Some(v) => Set(v), None => Set(expr))
fn parse_field(field: &syn::Field) -> Result<Option<IntoActiveModelField>, Error> {
    let ident = field.ident.as_ref().unwrap().clone();
    // Default expression for this field
    let mut default_expr: Option<syn::Expr> = None;

    for attr in field.attrs.iter() {
        if !attr.path().is_ident("sea_orm") {
            continue;
        }

        // Parse the attribute arguments: #[sea_orm(...)]
        if let Ok(list) = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated) {
            for meta in list.iter() {
                // Check for ignore/skip: #[sea_orm(ignore)] or #[sea_orm(skip)]
                if meta.exists("ignore") || meta.exists("skip") {
                    return Ok(None);
                }
                // Check for bare default: #[sea_orm(default)]
                if meta.exists("default") {
                    if default_expr.is_some() {
                        return Err(Error::Syn(syn::Error::new_spanned(
                            meta,
                            "duplicate `default` attribute",
                        )));
                    }
                    let expr: syn::Expr = syn::parse_quote!(::core::default::Default::default());
                    default_expr = Some(expr);
                    continue; // Skip next default check
                }
                // Check for default value: #[sea_orm(default = "expr")]
                if let Some(expr_str) = meta.get_as_kv("default") {
                    // Error on duplicate `default`
                    if default_expr.is_some() {
                        return Err(Error::Syn(syn::Error::new_spanned(
                            meta,
                            "duplicate `default` attribute",
                        )));
                    }
                    // Parse the expression string into a syn::Expr
                    let expr = syn::parse_str::<syn::Expr>(&expr_str).map_err(Error::Syn)?;
                    default_expr = Some(expr);
                }
            }
        }
    }

    // Finnaly match and return appropriate field type
    if let Some(expr) = default_expr {
        Ok(Some(IntoActiveModelField::WithDefault { ident, expr }))
    } else {
        Ok(Some(IntoActiveModelField::Normal(ident)))
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
