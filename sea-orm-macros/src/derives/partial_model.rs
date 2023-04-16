use heck::ToSnakeCase;
use heck::ToUpperCamelCase;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use quote::quote_spanned;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::Expr;

use syn::Meta;

use self::util::GetAsKVMeta;

#[derive(Debug)]
enum Error {
    InputNotStruct,
    EntityNotSpecific,
    BothFromColAndFromExpr(Span),
    Syn(syn::Error),
}
#[derive(Debug, PartialEq, Eq)]
enum ColumnAs {
    /// column in the model
    Col(syn::Ident),
    /// alias from a column in model
    ColAlias { col: syn::Ident, field: String },
    /// from a expr
    Expr { expr: syn::Expr, field_name: String },
}

struct DerivePartialModel {
    entity_ident: Option<syn::Ident>,
    ident: syn::Ident,
    fields: Vec<ColumnAs>,
}

impl DerivePartialModel {
    fn new(input: syn::DeriveInput) -> Result<Self, Error> {
        let syn::Data::Struct(syn::DataStruct{fields:syn::Fields::Named(syn::FieldsNamed{named:fields,..}),..},..)= input.data else{
            return Err(Error::InputNotStruct);
        };

        let mut entity_ident = None;

        for attr in input.attrs.iter() {
            if let Some(ident) = attr.path.get_ident() {
                if ident != "sea_orm" {
                    continue;
                }
            } else {
                continue;
            }

            if let Ok(list) = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated) {
                for meta in list {
                    entity_ident = meta
                        .get_as_kv("entity")
                        .map(|s| syn::parse_str::<syn::Ident>(&s).map_err(Error::Syn))
                        .transpose()?;
                }
            }
        }

        let mut column_as_list = Vec::with_capacity(fields.len());

        for field in fields {
            let field_span = field.span();

            let mut from_col = None;
            let mut from_expr = None;

            for attr in field.attrs.iter() {
                if !attr.path.is_ident("sea_orm") {
                    continue;
                }

                if let Ok(list) = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                {
                    for meta in list.iter() {
                        from_col = meta
                            .get_as_kv("from_col")
                            .map(|s| format_ident!("{}", s.to_upper_camel_case()));
                        from_expr = meta
                            .get_as_kv("from_expr")
                            .map(|s| syn::parse_str::<Expr>(&s).map_err(Error::Syn))
                            .transpose()?;
                    }
                }
            }

            let field_name = field.ident.unwrap();

            let col_as = match (from_col, from_expr) {
                (None, None) => {
                    if entity_ident.is_none() {
                        return Err(Error::EntityNotSpecific);
                    }
                    ColumnAs::Col(format_ident!(
                        "{}",
                        field_name.to_string().to_upper_camel_case()
                    ))
                }
                (None, Some(expr)) => ColumnAs::Expr {
                    expr,
                    field_name: field_name.to_string(),
                },
                (Some(col), None) => {
                    if entity_ident.is_none() {
                        return Err(Error::EntityNotSpecific);
                    }

                    let field = field_name.to_string().to_snake_case();
                    ColumnAs::ColAlias { col, field }
                }
                (Some(_), Some(_)) => return Err(Error::BothFromColAndFromExpr(field_span)),
            };
            column_as_list.push(col_as);
        }

        Ok(Self {
            entity_ident,
            ident: input.ident,
            fields: column_as_list,
        })
    }

    fn expand(&self) -> syn::Result<TokenStream> {
        Ok(self.impl_partial_model_trait())
    }

    fn impl_partial_model_trait(&self) -> TokenStream {
        let select_ident = format_ident!("select");
        let DerivePartialModel {
            entity_ident,
            ident,
            fields,
        } = self;
        let select_col_code_gen = fields.iter().map(|col_as| match col_as {
            ColumnAs::Col(ident) => {
                let entity = entity_ident.as_ref().unwrap();
                let col_value = quote!( <#entity as sea_orm::EntityTrait>::Column:: #ident);
                quote!(let #select_ident =  sea_orm::SelectColumns::select_column(#select_ident, #col_value);)
            },
            ColumnAs::ColAlias { col, field } => {
                let entity = entity_ident.as_ref().unwrap();
                let col_value = quote!( <#entity as sea_orm::EntityTrait>::Column:: #col);
                quote!(let #select_ident =  sea_orm::SelectColumns::select_column_as(#select_ident, #col_value, #field);)
            },
            ColumnAs::Expr { expr, field_name } => {
                quote!(let #select_ident =  sea_orm::SelectColumns::select_column_as(#select_ident, #expr, #field_name);)
            },
        });

        quote! {
            #[automatically_derived]
            impl sea_orm::PartialModelTrait for #ident{
                fn select_cols<S: sea_orm::SelectColumns>(#select_ident: S) -> S{
                    #(#select_col_code_gen)*
                    #select_ident
                }
            }
        }
    }
}

pub fn expand_derive_partial_model(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let ident_span = input.ident.span();

    match DerivePartialModel::new(input) {
        Ok(partial_model) => partial_model.expand(),
        Err(Error::BothFromColAndFromExpr(span)) => Ok(quote_spanned! {
            span => compile_error!("you can only use one of `from_col` or `from_expr`");
        }),
        Err(Error::EntityNotSpecific) => Ok(quote_spanned! {
            ident_span => compile_error!("you need specific witch entity you are using")
        }),
        Err(Error::InputNotStruct) => Ok(quote_spanned! {
            ident_span => compile_error!("you can only derive DeriveModel on structs");
        }),
        Err(Error::Syn(err)) => Err(err),
    }
}

mod util {
    use syn::{Lit, Meta, MetaNameValue};

    pub(super) trait GetAsKVMeta {
        fn get_as_kv(&self, k: &str) -> Option<String>;
    }

    impl GetAsKVMeta for Meta {
        fn get_as_kv(&self, k: &str) -> Option<String> {
            let Meta::NameValue(MetaNameValue{path,lit:Lit::Str(lit),..}) = self else {
                return  None;
            };

            if path.is_ident(k) {
                Some(lit.value())
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod test {
    use quote::format_ident;
    use syn::DeriveInput;

    use crate::derives::partial_model::ColumnAs;

    use super::DerivePartialModel;

    #[cfg(test)]
    type StdResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[cfg(test)]
    const CODE_SNIPPET: &str = r#"
#[sea_orm(entity = "Entity")]
struct PartialModel{
    default_field: i32,
    #[sea_orm(from_col = "bar")]
    alias_field: i32,
    #[sea_orm(from_expr = "Expr::val(1).add(1)")]
    expr_field : i32
}
"#;
    #[test]
    fn test_load_macro_input() -> StdResult<()> {
        let input = syn::parse_str::<DeriveInput>(CODE_SNIPPET)?;

        let middle = DerivePartialModel::new(input).unwrap();

        assert_eq!(middle.entity_ident, Some(format_ident!("Entity")));
        assert_eq!(middle.ident, format_ident!("PartialModel"));
        assert_eq!(middle.fields.len(), 3);
        assert_eq!(
            middle.fields[0],
            ColumnAs::Col(format_ident!("DefaultField"))
        );
        assert_eq!(
            middle.fields[1],
            ColumnAs::ColAlias {
                col: format_ident!("Bar"),
                field: "alias_field".to_string()
            },
        );
        assert_eq!(
            middle.fields[2],
            ColumnAs::Expr {
                expr: syn::parse_str("Expr::val(1).add(1)").unwrap(),
                field_name: "expr_field".to_string()
            }
        );

        Ok(())
    }
}
