use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{LitStr, Lit};

enum Error {
    InputNotEnum,
    Syn(syn::Error),
    // TT(TokenStream),
}

struct Display {
    ident: syn::Ident,
    name: String,
    variants: Vec<DisplayVariant>,
}

struct DisplayVariant {
    ident: syn::Ident,
    display_value: Option<LitStr>,
}

impl Display {
    fn new(input: syn::DeriveInput) -> Result<Self, Error> {
        let ident = input.ident;

        let mut name = ident.to_string();

        
        let variant_vec = match input.data {
            syn::Data::Enum(syn::DataEnum { variants, .. }) => variants,
            _ => return Err(Error::InputNotEnum),
        };

        
        let mut variants = Vec::new();
        for variant in variant_vec {
            let mut display_value = None;
            let variant_span = variant.ident.span();
            for attr in variant.attrs.iter() {
                if !attr.path().is_ident("sea_orm") {
                    continue;
                }
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("display_value") {
                        display_value = Some(meta.value()?.parse::<LitStr>()?);
                    } else {let _other_value = Some(meta.value()?.parse::<Lit>()?);}

                    Ok(())
                })
                .map_err(Error::Syn)?;
            }

            
            variants.push(DisplayVariant {
                ident: variant.ident,
                display_value,
            });
        }
        Ok(Display {
            ident,
            name,
            variants,
        })
    }

    fn expand(&self) -> syn::Result<TokenStream> {
        let expanded_impl_active_enum = self.impl_active_enum();

        Ok(expanded_impl_active_enum)
    }

    fn impl_active_enum(&self) -> TokenStream {
        let Self {
            ident,
            name,
            variants
        } = self;

        let variant_idents: Vec<syn::Ident> = variants
            .iter()
            .map(|variant| variant.ident.clone())
            .collect();

        let variant_display: Vec<TokenStream> = variants
            .iter()
            .map(|variant| {
                if let Some(display_value) = &variant.display_value {
                    let string = display_value.value();
                    quote! { #string }
                } else {
                    let ident = &variant.ident;
                    quote! { #ident }
                }
            })
            .collect();

        quote!(
            impl #ident {
                fn to_display_value(&self) -> String {
                match self {
                    #( Self::#variant_idents => #variant_display, )*
                    }
                    .to_owned()
                }
            }

            #[automatically_derived]
            impl std::fmt::Display for #ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let v: sea_orm::sea_query::Value = Self::to_display_value(&self).into();
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
            ident_span => compile_error!("you can only derive activeenum_Display on enums");
        }),
        // Err(Error::TT(token_stream)) => Ok(token_stream),
        Err(Error::Syn(e)) => Err(e),
    }
}
