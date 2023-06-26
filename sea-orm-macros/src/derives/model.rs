use super::util::{escape_rust_keyword, trim_starting_raw_identifier};
use darling::{ast, FromDeriveInput, FromField};
use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(sea_orm), supports(struct_named), allow_unknown_fields)]
pub struct DeriveModel {
    ident: syn::Ident,
    #[darling(default = "default_entity_ident")]
    entity: syn::Ident,
    data: ast::Data<(), DeriveModelField>,
}

#[derive(Debug, FromField)]
#[darling(attributes(sea_orm), allow_unknown_fields)]
struct DeriveModelField {
    ident: Option<syn::Ident>,
    enum_name: Option<syn::Ident>,
    #[darling(default)]
    ignore: bool,
}

fn default_entity_ident() -> syn::Ident {
    format_ident!("Entity")
}

impl ToTokens for DeriveModel {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let DeriveModel {
            ident,
            data,
            entity,
        } = self;

        let fields: Vec<_> = data
            .as_ref()
            .take_struct()
            .expect("Should never be enum")
            .fields
            .into_iter()
            .collect();

        let field_idents: Vec<_> = fields
            .iter()
            .map(|field| field.ident.as_ref().unwrap())
            .collect();
        let field_enum_names: Vec<_> = fields
            .iter()
            .map(|field| match field.enum_name.as_ref() {
                Some(enum_name) => enum_name.clone(),
                None => {
                    let ident = field.ident.as_ref().unwrap().to_string();
                    let ident = trim_starting_raw_identifier(ident).to_upper_camel_case();
                    let ident = escape_rust_keyword(ident);
                    format_ident!("{}", &ident)
                }
            })
            .collect();
        let field_ignores: Vec<_> = fields.iter().map(|field| field.ignore).collect();

        let field_values: Vec<_> = field_enum_names.iter()
                .zip(&field_ignores)
                .map(|(enum_name, ignore)| {
                    if *ignore {
                        quote! {
                            Default::default()
                        }
                    } else {
                        quote! {
                            row.try_get(pre, sea_orm::IdenStatic::as_str(&<<Self as sea_orm::ModelTrait>::Entity as sea_orm::entity::EntityTrait>::Column::#enum_name).into())?
                        }
                    }
                })
                .collect();

        tokens.extend([quote!(
            #[automatically_derived]
            impl sea_orm::FromQueryResult for #ident {
                fn from_query_result(row: &sea_orm::QueryResult, pre: &str) -> std::result::Result<Self, sea_orm::DbErr> {
                    Ok(Self {
                        #(#field_idents: #field_values,)*
                    })
                }
            }
        )]);

        let field_idents: Vec<_> = field_idents
            .iter()
            .zip(&field_ignores)
            .filter_map(|(ident, ignore)| if *ignore { None } else { Some(ident) })
            .collect();
        let field_enum_names: Vec<_> = field_enum_names
            .iter()
            .zip(&field_ignores)
            .filter_map(|(ident, ignore)| if *ignore { None } else { Some(ident) })
            .collect();

        let missing_field_msg = format!("field does not exist on {ident}");

        tokens.extend([quote!(
            #[automatically_derived]
            impl sea_orm::ModelTrait for #ident {
                type Entity = #entity;

                fn get(&self, c: <Self::Entity as sea_orm::entity::EntityTrait>::Column) -> sea_orm::Value {
                    match c {
                        #(<Self::Entity as sea_orm::entity::EntityTrait>::Column::#field_enum_names => self.#field_idents.clone().into(),)*
                        _ => panic!(#missing_field_msg),
                    }
                }

                fn set(&mut self, c: <Self::Entity as sea_orm::entity::EntityTrait>::Column, v: sea_orm::Value) {
                    match c {
                        #(<Self::Entity as sea_orm::entity::EntityTrait>::Column::#field_enum_names => self.#field_idents = v.unwrap(),)*
                        _ => panic!(#missing_field_msg),
                    }
                }
            }
        )]);
    }
}

/// Method to derive an ActiveModel
pub fn expand_derive_model(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let token = DeriveModel::from_derive_input(&input)
        .map(|derive_model| quote!(#derive_model))
        .unwrap();
    Ok(token)
}

#[test]
fn parse_derive_model() {
    let input = r#"
        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "hello")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            #[sea_orm(enum_name = "One1")]
            pub one: i32,
            pub two: i32,
            #[sea_orm(enum_name = "Three3")]
            pub three: i32,
            #[sea_orm(ignore)]
            pub cake_id: Option<i32>,
        }
    "#;

    let parsed = syn::parse_str(input).unwrap();
    let derive_model = DeriveModel::from_derive_input(&parsed).unwrap();
    let tokens = quote!(#derive_model);

    println!("INPUT:\n");
    println!("{input:#?}\n");
    println!("PARSED AS:\n");
    println!("{derive_model:#?}\n");
    println!("EMITS:\n");
    println!("{tokens}\n");

    // panic!() // UNCOMMENT this to force it panic and print above to console
}

#[test]
#[should_panic]
fn parse_derive_model_enum() {
    let input = r#"
        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "hello")]
        pub enum Model {
            #[sea_orm(primary_key)]
            Id
        }
    "#;

    let parsed = syn::parse_str(input).unwrap();
    let derive_model = DeriveModel::from_derive_input(&parsed).unwrap();
}
