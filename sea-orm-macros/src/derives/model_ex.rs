use super::attributes::compound_attr;
use super::entity_loader::{EntityLoaderField, EntityLoaderSchema, expand_entity_loader};
use super::util::is_compound_field;
use heck::ToUpperCamelCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    Attribute, Data, Fields, ItemStruct, LitStr, Meta, parse_quote, punctuated::Punctuated,
    token::Comma,
};

pub fn expand_sea_orm_model(input: ItemStruct) -> syn::Result<TokenStream> {
    let model = input.ident;
    let model_attrs = input.attrs;
    let vis = input.vis;
    let all_fields = input.fields;

    let model_ex = Ident::new(&format!("{}Ex", model), model.span());
    let mut model_ex_attrs = model_attrs.clone();
    for attr in &mut model_ex_attrs {
        if attr.path().is_ident("derive") {
            if let Meta::List(list) = &mut attr.meta {
                let mut new_list: Punctuated<_, Comma> = Punctuated::new();

                list.parse_nested_meta(|meta| {
                    if meta.path.is_ident("Eq") {
                        // skip
                    } else if meta.path.is_ident("DeriveEntityModel") {
                        // replace macro
                        new_list.push(parse_quote!(DeriveModelEx));
                    } else {
                        new_list.push(meta.path);
                    }

                    Ok(())
                })?;

                *attr = parse_quote!(#[derive( #new_list )]);
            }
        }
    }

    let model_fields: Vec<_> = all_fields
        .iter()
        .filter(|field| {
            let field_type = &field.ty;
            let field_type = quote! { #field_type }
                .to_string() // e.g.: "Option < String >"
                .replace(' ', ""); // Remove spaces

            !is_compound_field(&field_type)
        })
        .collect();

    Ok(quote! {
        #(#model_attrs)*
        #vis struct #model {
            #(#model_fields),*
        }

        #(#model_ex_attrs)*
        #vis struct #model_ex #all_fields
    })
}

pub fn expand_derive_model_ex(data: Data, attrs: Vec<Attribute>) -> syn::Result<TokenStream> {
    let mut table_name = None;
    let mut impl_related = Vec::new();
    let mut entity_loader_schema = EntityLoaderSchema::default();

    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("sea_orm"))
        .try_for_each(|attr| {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("table_name") {
                    table_name = Some(meta.value()?.parse::<LitStr>()?);
                }
                Ok(())
            })
        })?;

    if let Data::Struct(item_struct) = data {
        if let Fields::Named(fields) = item_struct.fields {
            for field in fields.named {
                if let Some(ident) = &field.ident {
                    let field_type = &field.ty;
                    let field_type = quote! { #field_type }
                        .to_string() // e.g.: "Option < String >"
                        .replace(' ', ""); // Remove spaces

                    if is_compound_field(&field_type) {
                        if field_type.starts_with("BelongsTo<") || field_type.starts_with("HasOne<")
                        {
                            entity_loader_schema.fields.push(EntityLoaderField {
                                is_one: true,
                                field: ident.clone(),
                                entity: extract_compound_entity(&field_type).to_owned(),
                            });
                        } else if field_type.starts_with("HasMany<") {
                            entity_loader_schema.fields.push(EntityLoaderField {
                                is_one: false,
                                field: ident.clone(),
                                entity: extract_compound_entity(&field_type).to_owned(),
                            });
                        }
                        if let Ok(attrs) = compound_attr::SeaOrm::from_attributes(&field.attrs) {
                            impl_related.push((attrs, field_type));
                        }
                    }
                }
            }
        }
    }

    let mut relation_enum_variants: Punctuated<_, Comma> = Punctuated::new();

    let related_def = {
        let mut ts = TokenStream::new();

        for (attrs, field_type) in impl_related.iter() {
            if let Some(var) = relation_enum_variant(attrs, field_type) {
                relation_enum_variants.push(var);
            }
            ts.extend(impl_related_trait(attrs, field_type, &table_name)?);
        }

        ts
    };

    let relation_enum = quote! {
        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {
            #relation_enum_variants
        }
    };

    let entity_loader = expand_entity_loader(entity_loader_schema);

    Ok(quote! {
        #relation_enum

        #related_def
    })

    // #entity_loader
}

fn relation_enum_variant(attr: &compound_attr::SeaOrm, ty: &str) -> Option<TokenStream> {
    let related_entity = extract_compound_entity(ty);
    let related_enum = Ident::new(
        &infer_relation_name_from_entity(related_entity).to_upper_camel_case(),
        Span::call_site(),
    );
    if ty.starts_with("BelongsTo<") {
        let from = format!(
            "Column::{}",
            attr.from
                .as_ref()
                .expect("Must specify `from` and `to` on BelongsTo relation")
                .value()
        );
        let to = format!(
            "{}::Column::{}",
            related_entity.trim_end_matches("::Entity"),
            attr.to
                .as_ref()
                .expect("Must specify `from` and `to` on BelongsTo relation")
                .value()
        );
        let belongs_to = Ident::new("belongs_to", Span::call_site());
        let mut extra: Punctuated<_, Comma> = Punctuated::new();
        if let Some(on_update) = &attr.on_update {
            let tag = Ident::new("on_update", Span::call_site());
            extra.push(quote!(#tag = #on_update))
        }
        if let Some(on_delete) = &attr.on_delete {
            let tag = Ident::new("on_delete", Span::call_site());
            extra.push(quote!(#tag = #on_delete))
        }

        Some(quote! {
            #[sea_orm(#belongs_to = #related_entity, from = #from, to = #to, #extra)]
            #related_enum
        })
    } else if ty.starts_with("HasMany<") && attr.via.is_none() {
        // skip junction relation

        let has_many = Ident::new("has_many", Span::call_site());

        Some(quote! {
            #[sea_orm(#has_many = #related_entity)]
            #related_enum
        })
    } else if ty.starts_with("HasOne<") {
        let has_one = Ident::new("has_one", Span::call_site());

        Some(quote! {
            #[sea_orm(#has_one = #related_entity)]
            #related_enum
        })
    } else {
        None
    }
}

fn impl_related_trait(
    attr: &compound_attr::SeaOrm,
    ty: &str,
    table_name: &Option<LitStr>,
) -> syn::Result<TokenStream> {
    if attr.relation.is_some() {
        let related_entity = extract_compound_entity(ty);
        let related_enum = infer_relation_name_from_entity(related_entity).to_upper_camel_case();
        let related_entity: TokenStream = related_entity.parse().unwrap();

        if let Some(via) = &attr.via {
            let via = via.value();
            let mut junction = via.as_str();
            let via_related = table_name
                .as_ref()
                .map(|v| v.value().to_upper_camel_case())
                .unwrap_or_default();
            let mut via_related = via_related.as_str();
            if let Some((prefix, suffix)) = via.split_once("::") {
                junction = prefix;
                via_related = suffix;
            }
            if via_related.is_empty() {
                return Err(syn::Error::new_spanned(
                    attr.via.as_ref().unwrap(),
                    "Please provide via in `my_entity::RelationVariant`",
                ));
            }
            let junction = Ident::new(junction, Span::call_site());
            let relation_def: TokenStream =
                format!("Relation::{related_enum}.def()").parse().unwrap();
            let via_relation_def: TokenStream =
                format!("Relation::{via_related}.def()").parse().unwrap();

            Ok(quote! {
                impl Related<#related_entity> for Entity {
                    fn to() -> RelationDef {
                        super::#junction::#relation_def
                    }
                    fn via() -> Option<RelationDef> {
                        Some(super::#junction::#via_relation_def.rev())
                    }
                }
            })

            // #[sea_orm(relation, via = "cakes_bakers::Cake")]
            // impl Related<super::baker::Entity> for Entity {
            //     fn to() -> RelationDef {
            //         super::cakes_bakers::Relation::Baker.def()
            //     }
            //     fn via() -> Option<RelationDef> {
            //         Some(super::cakes_bakers::Relation::Cake.def().rev())
            //     }
            // }
        } else {
            let relation_def: TokenStream =
                format!("Relation::{related_enum}.def()").parse().unwrap();

            Ok(quote! {
                impl Related<#related_entity> for Entity {
                    fn to() -> RelationDef {
                        #relation_def
                    }
                }
            })

            // #[sea_orm(relation)]
            // impl Related<super::bakery::Entity> for Entity {
            //     fn to() -> RelationDef {
            //         Relation::Bakery.def()
            //     }
            // }
        }
    } else {
        Ok(quote!())
    }
}

fn extract_compound_entity(ty: &str) -> &str {
    if ty.starts_with("BelongsTo<") {
        &ty["BelongsTo<".len()..(ty.len() - 1)]
    } else if ty.starts_with("HasMany<") {
        &ty["HasMany<".len()..(ty.len() - 1)]
    } else if ty.starts_with("HasOne<") {
        &ty["HasOne<".len()..(ty.len() - 1)]
    } else {
        panic!("`relation` attribute applied to non compound type: {ty}")
    }
}

fn infer_relation_name_from_entity(s: &str) -> &str {
    let s = s.trim_end_matches("::Entity");
    if let Some((_, suffix)) = s.rsplit_once("::") {
        return suffix;
    }
    s
}
