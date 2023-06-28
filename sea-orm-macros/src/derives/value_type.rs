use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Data, Expr, Fields, Lit, LitStr, Type, Token};

enum Error {
    Syn(syn::Error),
    TT(TokenStream),
}
struct DeriveValueType {
    name: syn::Ident,
    ty: Type,
    column_type: TokenStream,
}

impl DeriveValueType {
    pub fn new(input: syn::DeriveInput) -> Result<Self, Error> {
        let dat = input.data;
        let fields: Option<syn::punctuated::Punctuated<syn::Field, syn::token::Comma>> = match dat {
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Unnamed(syn::FieldsUnnamed { unnamed, .. }),
                ..
            }) => Some(unnamed),
            _ => None,
        };

        let field = fields
            .expect("This derive accept only struct")
            .first()
            .expect("The struct should contain one value field")
            .to_owned();

        let ty = field.clone().ty;
        let name = input.ident;
        let mut column_type = quote!("abc");
        let mut sql_type = None;

        // search for #[sea_orm(primary_key, auto_increment = false, column_type = "String(Some(255))", default_value = "new user", default_expr = "gen_random_uuid()", column_name = "name", enum_name = "Name", nullable, indexed, unique)]
        for attr in field.attrs.iter() {
            if !attr.path().is_ident("sea_orm") {
                continue;
            }

            // single param
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("column_type") {
                    let lit = meta.value()?.parse()?;
                    if let Lit::Str(litstr) = lit {
                        let ty: TokenStream = syn::parse_str(&litstr.value())?;
                        sql_type = Some(ty);
                    } else {
                        return Err(meta.error(format!("Invalid column_type {:?}", lit)));
                    }
                } else {
                    // Reads the value expression to advance the parse stream.
                    // Some parameters, such as `primary_key`, do not have any value,
                    // so ignoring an error occurred here.
                    let _: Option<Expr> = meta.value().and_then(|v| v.parse()).ok();
                }

                Ok(())
            }).expect("msg");
        }

        ty = match sql_type {
            Some(t) => match t.to_string().as_str() { 
                "char" => Char(None),
                "String" | "&str" => Type::Tuple(String(None)),
                "i8" => Type::Tuple(String(None)),
                "u8" => Type::Tuple(String(None)),
                "i16" => Type::Tuple(String(None)),
                "u16" => Type::Tuple(String(None)),
                "i32" => Type::Tuple(String(None)),
                "u32" => Type::Tuple(String(None)),
                "i64" => Type::Tuple(String(None)),
                "u64" => Type::Tuple(String(None)),
                "f32" => Type::Tuple(String(None)),
                "f64" => Type::Tuple(String(None)),
                "bool" => Type::Tuple(String(None)),
                "Date" | "NaiveDate" => Type::Tuple(String(None)),
                "Time" | "NaiveTime" => Type::Tuple(String(None)),
                "DateTime" | "NaiveDateTime" => {
                    Type::Tuple(String(None))
                }
                "DateTimeUtc" | "DateTimeLocal" | "DateTimeWithTimeZone" => {
                    Type::Tuple(String(None))
                }
                "Uuid" => Type::Tuple(String(None)),
                "Json" => Type::Tuple(String(None)),
                "Decimal" => Type::Tuple(String(None)),
                "Vec<u8>" => {
                    Type::Tuple(String(None))
                },
            None => ty
        };

        Ok(DeriveValueType {
            name,
            ty,
            column_type,
        })
    }

    fn expand(&self) -> syn::Result<TokenStream> {
        let expanded_impl_value_type: TokenStream = self.impl_value_type();
        Ok(expanded_impl_value_type)
    }

    fn impl_value_type(&self) -> TokenStream {
        let name = &self.name;
        let mut ty = &self.ty;
        if &self.column_type.is_empty() {ty = &self.column_type}
        let column_type = &self.column_type;

        quote!(
            #[automatically_derived]
            impl From<#name> for Value {
                fn from(source: #name) -> Self {
                    source.0.into()
                }
            }

            #[automatically_derived]
            impl sea_orm::TryGetable for #name {
                fn try_get_by<I: sea_orm::ColIdx>(res: &QueryResult, idx: I) -> Result<Self, sea_orm::TryGetError> {
                    <#ty as sea_orm::TryGetable>::try_get_by(res, idx).map(|v| #name(v))
                }
            }

            #[automatically_derived]
            impl sea_query::ValueType for #name {
                fn try_from(v: Value) -> Result<Self, sea_query::ValueTypeErr> {
                    <#ty as sea_query::ValueType>::try_from(v).map(|v| #name(v))
                }

                fn type_name() -> String {
                    stringify!(#name).to_owned()
                }

                fn array_type() -> sea_orm::sea_query::ArrayType {
                    <#ty as sea_orm::sea_query::ValueType>::array_type()
                }

                fn column_type() -> sea_orm::sea_query::ColumnType {
                    <#ty as sea_orm::sea_query::ValueType>::column_type()
                }
            }
        )
    }
}

pub fn expand_derive_value_type(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    match DeriveValueType::new(input) {
        Ok(model) => model.expand(),
        Err(Error::TT(token_stream)) => Ok(token_stream),
        Err(Error::Syn(e)) => Err(e),
    }
}
