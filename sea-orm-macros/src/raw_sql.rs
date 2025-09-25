use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Ident, LitStr, Token,
    parse::{Parse, ParseStream},
};

struct CallArgs {
    backend: Ident,
    _comma: Token![,],
    sql_string: LitStr,
}

impl Parse for CallArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(CallArgs {
            backend: input.parse()?,
            _comma: input.parse()?,
            sql_string: input.parse()?,
        })
    }
}

pub fn expand(input: proc_macro::TokenStream) -> syn::Result<TokenStream> {
    let CallArgs {
        backend,
        sql_string,
        ..
    } = syn::parse(input)?;

    let builder = match backend.to_string().as_str() {
        "MySql" => quote!(MysqlQueryBuilder),
        "Postgres" => quote!(PostgresQueryBuilder),
        "Sqlite" => quote!(SqliteQueryBuilder),
        backend => panic!("Unsupported backend {backend}"),
    };

    Ok(quote! {{
        use sea_orm::sea_query;

        let query = sea_query::raw_query!(#builder, #sql_string);

        sea_orm::Statement {
            sql: query.sql,
            values: Some(query.values),
            db_backend: sea_orm::DbBackend::#backend,
        }
    }})
}
