use bae::FromAttributes;
use heck::SnakeCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{Attribute, Lit, LitStr, Result, Visibility};

#[derive(Default, FromAttributes)]
struct Table {
    schema: Option<Lit>,
    name: Option<Lit>,
}

pub(crate) fn expand_entity(
    attrs: &[Attribute],
    vis: &Visibility,
    ident: &Ident,
) -> Result<TokenStream> {
    let table_attr = Table::try_from_attributes(attrs)?.unwrap_or_default();
    let table_name = table_attr.name.unwrap_or_else(|| {
        Lit::Str(LitStr::new(
            &(ident.to_string().to_snake_case() + "s"),
            Span::call_site(),
        ))
    });
    let schema_name_expanded = table_attr
        .schema
        .map(|schema| quote!(Some(#schema)))
        .unwrap_or_else(|| quote!(None));

    let entity_ident = format_ident!("{}Entity", ident);
    let column_ident = format_ident!("{}Column", ident);
    let primary_key_ident = format_ident!("{}PrimaryKey", ident);
    let relation_ident = format_ident!("{}Relation", ident);

    let expanded = quote!(
        #[derive(Copy, Clone, Default, Debug)]
        #vis struct #entity_ident;

        impl sea_orm::EntityName for #entity_ident {
            fn schema_name(&self) -> Option<&str> {
                #schema_name_expanded
            }

            fn table_name(&self) -> &str {
                #table_name
            }
        }

        impl sea_orm::Iden for #entity_ident {
            fn unquoted(&self, s: &mut dyn std::fmt::Write) {
                write!(s, "{}", <#entity_ident as sea_orm::IdenStatic>::as_str(self)).unwrap();
            }
        }

        impl sea_orm::IdenStatic for #entity_ident {
            fn as_str(&self) -> &str {
                <Self as sea_orm::EntityName>::table_name(self)
            }
        }

        impl sea_orm::entity::EntityTrait for #entity_ident {
            type Model = #ident;

            type Column = #column_ident;

            type PrimaryKey = #primary_key_ident;

            type Relation = #relation_ident;
        }
    );

    Ok(expanded)
}
