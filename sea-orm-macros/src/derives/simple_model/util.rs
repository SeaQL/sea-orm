use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use syn::{Attribute, Error, Path, PathSegment, Result, Type};

pub(crate) fn get_token_stream_derives(stream: TokenStream) -> TokenStream {
    match split_token_stream(stream, ',') {
        Ok(derives) => quote!(#[derive(#(#derives), *)]),
        Err(err) => err.to_compile_error(),
    }
}

pub(crate) fn get_token_stream_derives_with(
    stream: TokenStream,
    mut derives: Vec<TokenStream>,
) -> TokenStream {
    match split_token_stream(stream, ',') {
        Ok(mut stream_derives) => {
            derives.append(&mut stream_derives);
            quote!(#[derive(#(#derives), *)])
        }
        Err(err) => err.to_compile_error(),
    }
}

pub(crate) fn get_token_stream_attributes(stream: TokenStream) -> TokenStream {
    match split_token_stream(stream, ',') {
        Ok(attributes) => quote!(#(#[#attributes]) *),
        Err(err) => err.to_compile_error(),
    }
}

pub(crate) fn has_attribute(name: &str, attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path
            .segments
            .iter()
            .any(|segment| segment.ident == name)
    })
}

pub(crate) fn split_token_stream(stream: TokenStream, separator: char) -> Result<Vec<TokenStream>> {
    if let Some(token_tree) = stream.clone().into_iter().next() {
        let items = match token_tree {
            TokenTree::Group(group) => group.stream().into_iter().into_iter().fold(
                vec![TokenStream::new()],
                |mut acc, ref attr_part| {
                    let is_separator = match attr_part {
                        TokenTree::Punct(punct) => punct.as_char() == separator,
                        _ => false,
                    };

                    if is_separator {
                        acc.push(TokenStream::new());
                    } else {
                        let last_token_stream = acc.last_mut().unwrap();
                        last_token_stream.extend_one(attr_part.clone());
                    }

                    acc
                },
            ),
            TokenTree::Ident(ident) => vec![TokenStream::from(TokenTree::Ident(ident))],
            _ => return Err(Error::new_spanned(stream, "malformed syntax")),
        };

        Ok(items)
    } else {
        Err(Error::new_spanned(stream, "malformed syntax"))
    }
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
