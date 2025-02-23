use heck::ToUpperCamelCase;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use quote::quote_spanned;
use syn::ext::IdentExt;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::Expr;

use syn::Meta;
use syn::Type;

use super::from_query_result::{
    DeriveFromQueryResult, FromQueryResultItem, ItemType as FqrItemType,
};
use super::util::GetMeta;

#[derive(Debug)]
enum Error {
    InputNotStruct,
    EntityNotSpecified,
    NotSupportGeneric(Span),
    OverlappingAttributes(Span),
    Syn(syn::Error),
}

#[derive(Debug, PartialEq, Eq)]
enum ColumnAs {
    /// column in the model
    Col(syn::Ident),
    /// alias from a column in model
    ColAlias {
        col: syn::Ident,
        field: syn::Ident,
    },
    /// from an expr
    Expr {
        expr: syn::Expr,
        field: syn::Ident,
    },
    /// nesting another struct
    Nested {
        typ: Type,
        field: syn::Ident,
    },
    Skip(syn::Ident),
}

struct DerivePartialModel {
    entity: Option<syn::Type>,
    ident: syn::Ident,
    fields: Vec<ColumnAs>,
    from_query_result: bool,
}

impl DerivePartialModel {
    fn new(input: syn::DeriveInput) -> Result<Self, Error> {
        if !input.generics.params.is_empty() {
            return Err(Error::NotSupportGeneric(input.generics.params.span()));
        }

        let syn::Data::Struct(
            syn::DataStruct {
                fields: syn::Fields::Named(syn::FieldsNamed { named: fields, .. }),
                ..
            },
            ..,
        ) = input.data
        else {
            return Err(Error::InputNotStruct);
        };

        let mut entity = None;
        let mut from_query_result = false;

        for attr in input.attrs.iter() {
            if !attr.path().is_ident("sea_orm") {
                continue;
            }

            if let Ok(list) = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated) {
                for meta in list {
                    if let Some(s) = meta.get_as_kv("entity") {
                        entity = Some(syn::parse_str::<syn::Type>(&s).map_err(Error::Syn)?);
                    } else if meta.exists("from_query_result") {
                        from_query_result = true;
                    }
                }
            }
        }

        let mut column_as_list = Vec::with_capacity(fields.len());

        for field in fields {
            let field_span = field.span();

            let mut from_col = None;
            let mut from_expr = None;
            let mut nested = false;
            let mut skip = false;

            for attr in field.attrs.iter() {
                if !attr.path().is_ident("sea_orm") {
                    continue;
                }

                if let Ok(list) = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                {
                    for meta in list.iter() {
                        if meta.exists("skip") {
                            skip = true;
                        } else if meta.exists("nested") {
                            nested = true;
                        } else if let Some(s) = meta.get_as_kv("from_col") {
                            from_col = Some(format_ident!("{}", s.to_upper_camel_case()));
                        } else if let Some(s) = meta.get_as_kv("from_expr") {
                            from_expr = Some(syn::parse_str::<Expr>(&s).map_err(Error::Syn)?);
                        }
                    }
                }
            }

            let field_name = field.ident.unwrap();

            let col_as = match (from_col, from_expr, nested) {
                (Some(col), None, false) => {
                    if entity.is_none() {
                        return Err(Error::EntityNotSpecified);
                    }

                    ColumnAs::ColAlias {
                        col,
                        field: field_name,
                    }
                }
                (None, Some(expr), false) => ColumnAs::Expr {
                    expr,
                    field: field_name,
                },
                (None, None, true) => ColumnAs::Nested {
                    typ: field.ty,
                    field: field_name.unraw(),
                },
                (None, None, false) => {
                    if entity.is_none() {
                        return Err(Error::EntityNotSpecified);
                    }
                    if skip {
                        ColumnAs::Skip(field_name)
                    } else {
                        ColumnAs::Col(field_name)
                    }
                }
                (_, _, _) => return Err(Error::OverlappingAttributes(field_span)),
            };
            column_as_list.push(col_as);
        }

        Ok(Self {
            entity,
            ident: input.ident,
            fields: column_as_list,
            from_query_result,
        })
    }

    fn expand(&self) -> syn::Result<TokenStream> {
        let impl_partial_model = self.impl_partial_model();

        let impl_from_query_result = if self.from_query_result {
            DeriveFromQueryResult {
                ident: self.ident.clone(),
                generics: Default::default(),
                fields: self
                    .fields
                    .iter()
                    .map(|col_as| FromQueryResultItem {
                        typ: match col_as {
                            ColumnAs::Nested { .. } => FqrItemType::Nested,
                            ColumnAs::Skip(_) => FqrItemType::Skip,
                            _ => FqrItemType::Flat,
                        },
                        ident: match col_as {
                            ColumnAs::Col(field) => field,
                            ColumnAs::ColAlias { field, .. } => field,
                            ColumnAs::Expr { field, .. } => field,
                            ColumnAs::Nested { field, .. } => field,
                            ColumnAs::Skip(field) => field,
                        }
                        .to_owned(),
                        alias: None,
                    })
                    .collect(),
            }
            .impl_from_query_result(true)
        } else {
            quote!()
        };

        Ok(quote! {
            #impl_partial_model
            #impl_from_query_result
        })
    }

    fn impl_partial_model(&self) -> TokenStream {
        let select_ident = format_ident!("select");
        let DerivePartialModel {
            entity,
            ident,
            fields,
            ..
        } = self;
        let select_col_code_gen = fields.iter().map(|col_as| match col_as {
            ColumnAs::Col(ident) => {
                let entity = entity.as_ref().unwrap();
                let uppercase_ident = format_ident!(
                    "{}",
                    ident.to_string().to_upper_camel_case()
                );
                let col_value = quote!( <#entity as sea_orm::EntityTrait>::Column:: #uppercase_ident);
                let ident_stringified = ident.unraw().to_string();
                quote!(let #select_ident =
                       if let Some(prefix) = pre {
                           let ident = format!("{prefix}{}", #ident_stringified);
                           sea_orm::SelectColumns::select_column_as(#select_ident, #col_value, ident)
                       } else {
                           sea_orm::SelectColumns::select_column_as(#select_ident, #col_value, #ident_stringified)
                       };
                )
            },
            ColumnAs::ColAlias { col, field } => {
                let field = field.to_string();
                let entity = entity.as_ref().unwrap();
                let col_value = quote!( <#entity as sea_orm::EntityTrait>::Column:: #col);
                quote!(let #select_ident =
                       if let Some(prefix) = pre {
                           let ident = format!("{prefix}{}", #field);
                           sea_orm::SelectColumns::select_column_as(#select_ident, #col_value, ident)
                       } else {
                           sea_orm::SelectColumns::select_column_as(#select_ident, #col_value, #field)
                       };
                )
            },
            ColumnAs::Expr { expr, field } => {
                let field = field.to_string();
                quote!(let #select_ident =
                       if let Some(prefix) = pre {
                           let ident = format!("{prefix}{}", #field);
                           sea_orm::SelectColumns::select_column_as(#select_ident, #expr, ident)
                       } else {
                           sea_orm::SelectColumns::select_column_as(#select_ident, #expr, #field)
                       };
                )
            },
            ColumnAs::Nested { typ, field } => {
                let field = field.to_string();
                quote!(let #select_ident =
                       <#typ as sea_orm::PartialModelTrait>::select_cols_nested(#select_ident,
                                Some(&if let Some(prefix) = pre {
                                          format!("{prefix}{}_", #field) } 
                                      else {
                                          format!("{}_", #field)
                                      }
                                ));
                )
            },
            ColumnAs::Skip(_) => quote!(),
        });

        quote! {
            #[automatically_derived]
            impl sea_orm::PartialModelTrait for #ident {
                fn select_cols<S: sea_orm::SelectColumns>(#select_ident: S) -> S {
                    Self::select_cols_nested(#select_ident, None)
                }

                fn select_cols_nested<S: sea_orm::SelectColumns>(#select_ident: S, pre: Option<&str>) -> S {
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
        Err(Error::NotSupportGeneric(span)) => Ok(quote_spanned! {
            span => compile_error!("you can only derive `DerivePartialModel` on concrete struct");
        }),
        Err(Error::OverlappingAttributes(span)) => Ok(quote_spanned! {
            span => compile_error!("you can only use one of `from_col`, `from_expr`, `nested`");
        }),
        Err(Error::EntityNotSpecified) => Ok(quote_spanned! {
            ident_span => compile_error!("you need specific which entity you are using")
        }),
        Err(Error::InputNotStruct) => Ok(quote_spanned! {
            ident_span => compile_error!("you can only derive `DerivePartialModel` on named struct");
        }),
        Err(Error::Syn(err)) => Err(err),
    }
}

#[cfg(test)]
mod test {
    use quote::format_ident;
    use syn::{parse_str, DeriveInput, Type};

    use crate::derives::partial_model::ColumnAs;

    use super::DerivePartialModel;

    type StdResult<T> = Result<T, Box<dyn std::error::Error>>;

    const CODE_SNIPPET_1: &str = r#"
        #[sea_orm(entity = "Entity")]
        struct PartialModel {
            default_field: i32,
            #[sea_orm(from_col = "bar")]
            alias_field: i32,
            #[sea_orm(from_expr = "Expr::val(1).add(1)")]
            expr_field : i32
        }
        "#;

    #[test]
    fn test_load_macro_input_1() -> StdResult<()> {
        let input = parse_str::<DeriveInput>(CODE_SNIPPET_1)?;

        let middle = DerivePartialModel::new(input).unwrap();
        assert_eq!(middle.entity, Some(parse_str::<Type>("Entity").unwrap()));
        assert_eq!(middle.ident, format_ident!("PartialModel"));
        assert_eq!(middle.fields.len(), 3);
        assert_eq!(
            middle.fields[0],
            ColumnAs::Col(format_ident!("default_field"))
        );
        assert_eq!(
            middle.fields[1],
            ColumnAs::ColAlias {
                col: format_ident!("Bar"),
                field: format_ident!("alias_field"),
            },
        );
        assert_eq!(
            middle.fields[2],
            ColumnAs::Expr {
                expr: syn::parse_str("Expr::val(1).add(1)").unwrap(),
                field: format_ident!("expr_field"),
            }
        );
        assert_eq!(middle.from_query_result, false);

        Ok(())
    }

    const CODE_SNIPPET_2: &str = r#"
        #[sea_orm(entity = "MyEntity", from_query_result)]
        struct PartialModel {
            default_field: i32,
        }
        "#;

    #[test]
    fn test_load_macro_input_2() -> StdResult<()> {
        let input = parse_str::<DeriveInput>(CODE_SNIPPET_2)?;

        let middle = DerivePartialModel::new(input).unwrap();
        assert_eq!(middle.entity, Some(parse_str::<Type>("MyEntity").unwrap()));
        assert_eq!(middle.ident, format_ident!("PartialModel"));
        assert_eq!(middle.fields.len(), 1);
        assert_eq!(
            middle.fields[0],
            ColumnAs::Col(format_ident!("default_field"))
        );
        assert_eq!(middle.from_query_result, true);

        Ok(())
    }
}
