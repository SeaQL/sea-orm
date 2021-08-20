use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    AngleBracketedGenericArguments, Data, DataStruct, Field, Fields, GenericArgument,
    PathArguments, Type, TypePath,
};

fn option_type_to_inner_type(ty: &Type) -> Option<&Type> {
    let TypePath { path, .. } = if let Type::Path(path) = ty {
        path
    } else {
        return None;
    };

    let segment = path.segments.first()?;
    if segment.ident != "Option" {
        return None;
    }

    let generic_arg =
        if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
            &segment.arguments
        {
            args.first()?
        } else {
            return None;
        };

    let opt_ty = if let GenericArgument::Type(ty) = generic_arg {
        ty
    } else {
        return None;
    };

    let TypePath { path, .. } = if let Type::Path(path) = ty {
        path
    } else {
        return None;
    };

    let segment = path.segments.first()?;
    if segment.ident == "Option" {
        return option_type_to_inner_type(opt_ty);
    }

    Some(opt_ty)
}

pub fn expand_derive_partial_model(ident: Ident, data: Data) -> syn::Result<TokenStream> {
    let fields = match data {
        Data::Struct(DataStruct {
            fields: Fields::Named(named),
            ..
        }) => named.named,
        _ => {
            return Ok(quote_spanned! {
                ident.span() => compile_error!("you can only derive DeriveActiveModel on structs");
            })
        }
    };

    let field: Vec<_> = fields
        .clone()
        .into_iter()
        .filter_map(|Field { ident, .. }| ident)
        .collect();

    let name: Vec<_> = field.iter().map(|f| f.to_string()).collect();

    let ty: Vec<&Type> = fields
        .iter()
        .map(|Field { ty, .. }| option_type_to_inner_type(ty).unwrap_or(ty))
        .collect();

    Ok(quote!(
        #[derive(Clone, Default, Debug, PartialEq)]
        pub struct PartialModel {
            #(pub #field: std::option::Option<#ty>),*
        }

        impl FromQueryResult for PartialModel {
            fn from_query_result(res: &QueryResult, pre: &str) -> std::result::Result<Self, DbErr>
            where
                Self: Sized,
            {
                Ok(Self {
                    #(#field: res.try_get::<#ty>(pre, #name).ok()),*
                })
            }
        }
    ))
}
