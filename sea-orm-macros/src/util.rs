use heck::CamelCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{
    punctuated::Punctuated, spanned::Spanned, token::Comma, Attribute, Field, Generics, Path,
    PathSegment, Type,
};

pub(crate) fn has_attribute(name: &str, attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path
            .segments
            .iter()
            .any(|segment| segment.ident == name)
    })
}

pub(crate) fn expand_model_field_validation(
    ident: &Ident,
    mut generics: Generics,
    model_ident: &Ident,
    fields: &Punctuated<Field, Comma>,
) -> TokenStream {
    generics
        .lifetimes_mut()
        .into_iter()
        .for_each(|mut lifetime| {
            lifetime.lifetime.ident = format_ident!("_");
        });

    let checks = fields.into_iter().map(|field| {
        let fn_name = format_ident!(
            "_Assert{}{}",
            model_ident,
            field.ident.as_ref().unwrap().to_string().to_camel_case()
        );

        let ty = {
            let mut ty = option_type_to_inner_type(&field.ty)
                .map(Clone::clone)
                .unwrap_or_else(|| field.ty.clone());

            // Rename lifetimes to _
            if let Type::Reference(ref mut type_ref) = ty {
                type_ref.lifetime = type_ref.lifetime.clone().map(|mut lifetime| {
                    lifetime.ident = format_ident!("_");
                    lifetime
                });
            }

            ty
        };

        quote_spanned!(field.ty.span()=> impl #fn_name<#ty> for #ident#generics {})
    });

    quote!(
        #(#checks)*
    )
}

/// Returns in inner type of an `Option` `syn::Type`.
/// `None` is returned if the type provided is not wrapped in an `Option`, otherwise `Some` along with the inner type is returned.
pub(crate) fn option_type_to_inner_type(ty: &Type) -> Option<&Type> {
    fn extract_type_path(ty: &Type) -> Option<&Path> {
        match *ty {
            Type::Path(ref typepath) if typepath.qself.is_none() => Some(&typepath.path),
            _ => None,
        }
    }

    fn extract_option_segment(path: &Path) -> Option<&PathSegment> {
        let idents_of_path = path
            .segments
            .iter()
            .into_iter()
            .fold(String::new(), |mut acc, v| {
                acc.push_str(&v.ident.to_string());
                acc.push('|');
                acc
            });
        vec!["Option|", "std|option|Option|", "core|option|Option|"]
            .into_iter()
            .find(|s| idents_of_path == *s)
            .and_then(|_| path.segments.last())
    }

    extract_type_path(ty)
        .and_then(|path| extract_option_segment(path))
        .and_then(|path_seg| {
            let type_params = &path_seg.arguments;
            // It should have only on angle-bracketed param ("<String>"):
            match *type_params {
                syn::PathArguments::AngleBracketed(ref params) => params.args.first(),
                _ => None,
            }
        })
        .and_then(|generic_arg| match *generic_arg {
            syn::GenericArgument::Type(ref ty) => Some(ty),
            _ => None,
        })
}
