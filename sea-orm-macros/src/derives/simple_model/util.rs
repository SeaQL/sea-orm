use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use syn::{Attribute, Error, Result};

pub(crate) fn get_token_stream_derives(stream: TokenStream) -> TokenStream {
    match split_token_stream(stream, ',') {
        Ok(derives) => quote!(#[derive(#(#derives), *)]),
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
