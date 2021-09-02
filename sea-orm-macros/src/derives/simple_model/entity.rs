use bae::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Attribute, Lit, Result, Visibility};

#[derive(FromAttributes)]
struct Table {
    schema: Option<Lit>,
    name: Lit,
}

pub(crate) fn expand_entity(
    attrs: &[Attribute],
    vis: Visibility,
    ident: Ident,
) -> Result<TokenStream> {
    let table_attr = Table::from_attributes(attrs)?;
    let schema_name_expended = table_attr
        .schema
        .map(|schema| quote!(Some(#schema)))
        .unwrap_or_else(|| quote!(None));
    let table_name = table_attr.name;

    let entity_ident = format_ident!("{}Entity", ident);
    let column_ident = format_ident!("{}Column", ident);
    let primary_key_ident = format_ident!("{}PrimaryKey", ident);
    let relation_ident = format_ident!("{}Relation", ident);

    let expanded = quote!(
        #[derive(Copy, Clone, Default, Debug)]
        #vis struct #entity_ident;

        impl EntityName for #entity_ident {
            fn schema_name(&self) -> Option<&str> {
                #schema_name_expended
            }

            fn table_name(&self) -> &str {
                #table_name
            }
        }

        impl sea_orm::Iden for #entity_ident {
            fn unquoted(&self, s: &mut dyn std::fmt::Write) {
                write!(s, "{}", self.as_str()).unwrap();
            }
        }

        impl sea_orm::IdenStatic for #entity_ident {
            fn as_str(&self) -> &str {
                <Self as sea_orm::EntityName>::table_name(self)
            }
        }

        impl EntityTrait for #entity_ident {
            type Model = #ident;

            type Column = #column_ident;

            type PrimaryKey = #primary_key_ident;

            type Relation = #relation_ident;
        }
    );

    Ok(expanded)
}
