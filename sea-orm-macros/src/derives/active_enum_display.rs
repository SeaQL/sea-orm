use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{LitInt, LitStr};

enum Error {
    InputNotEnum,
    Syn(syn::Error),
}

struct Display {
    ident: syn::Ident,
    variants: Vec<DisplayVariant>,
}

struct DisplayVariant {
    ident: syn::Ident,
    display_value: TokenStream,
}

impl Display {
    fn new(input: syn::DeriveInput) -> Result<Self, Error> {
        let ident = input.ident;

        let variant_vec = match input.data {
            syn::Data::Enum(syn::DataEnum { variants, .. }) => variants,
            _ => return Err(Error::InputNotEnum),
        };

        let mut variants = Vec::new();
        for variant in variant_vec {
            let mut display_value = variant.ident.clone().to_string().to_token_stream();
            for attr in variant.attrs.iter() {
                if !attr.path().is_ident("sea_orm") {
                    continue;
                }
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("string_value") {
                        meta.value()?.parse::<LitStr>()?;
                    } else if meta.path.is_ident("num_value") {
                        meta.value()?.parse::<LitInt>()?;
                    } else if meta.path.is_ident("display_value") {
                        display_value = meta.value()?.parse::<LitStr>()?.to_token_stream();
                    } else {
                        return Err(meta.error(format!(
                            "Unknown attribute parameter found: {:?}",
                            meta.path.get_ident()
                        )));
                    }

                    Ok(())
                })
                .map_err(Error::Syn)?;
            }
            variants.push(DisplayVariant {
                ident: variant.ident,
                display_value,
            });
        }
        Ok(Display { ident, variants })
    }

    fn expand(&self) -> syn::Result<TokenStream> {
        let expanded_impl_active_enum = self.impl_active_enum();

        Ok(expanded_impl_active_enum)
    }

    fn impl_active_enum(&self) -> TokenStream {
        let Self { ident, variants } = self;

        let variant_idents: Vec<syn::Ident> = variants
            .iter()
            .map(|variant| variant.ident.clone())
            .collect();

        let variant_display: Vec<TokenStream> = variants
            .iter()
            .map(|variant| variant.display_value.to_owned())
            .collect();

        quote!(
            #[automatically_derived]
            impl std::fmt::Display for #ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let v = match self {
                        #( Self::#variant_idents => #variant_display, )*
                        }
                        .to_owned();
                    write!(f, "{}", v)
                }
            }
        )
    }
}

pub fn expand_derive_active_enum_display(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let ident_span = input.ident.span();

    match Display::new(input) {
        Ok(model) => model.expand(),
        Err(Error::InputNotEnum) => Ok(quote_spanned! {
            ident_span => compile_error!("you can only derive EnumDisplay on enums");
        }),
        Err(Error::Syn(e)) => Err(e),
    }
}
