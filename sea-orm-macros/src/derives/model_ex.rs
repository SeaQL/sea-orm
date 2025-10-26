use super::attributes::{column_attr, compound_attr};
use super::entity_loader::{EntityLoaderField, EntityLoaderSchema, expand_entity_loader};
use super::model::DeriveModel;
use super::util::{format_field_ident_ref, is_compound_field};
use heck::ToUpperCamelCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use std::collections::{BTreeMap, HashMap};
use syn::{
    Attribute, Data, Fields, ItemStruct, LitStr, Meta, Type, parse_quote, punctuated::Punctuated,
    token::Comma,
};

pub fn expand_sea_orm_model(input: ItemStruct, compact: bool) -> syn::Result<TokenStream> {
    let model = input.ident;
    let model_attrs = input.attrs;
    let vis = input.vis;
    let mut all_fields = input.fields;

    let model_ex = Ident::new(&format!("{model}Ex"), model.span());
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

    let compact_model = if compact {
        quote!(#[sea_orm(compact_model)])
    } else {
        quote!()
    };

    let mut model_fields = Vec::new();

    for field in all_fields.iter_mut() {
        let field_type = &field.ty;
        let field_type = quote! { #field_type }
            .to_string() // e.g.: "Option < String >"
            .replace(' ', ""); // Remove spaces

        if is_compound_field(&field_type) {
            let entity_path = extract_compound_entity(&field_type);
            if field_type.starts_with("Option<") || field_type.starts_with("HasOne<") {
                field.ty = syn::parse_str(&format!("HasOne < {entity_path} >"))?;
            } else {
                field.ty = syn::parse_str(&format!("HasMany < {entity_path} >"))?;
            }
        } else {
            model_fields.push(field);
        }
    }

    Ok(quote! {
        #(#model_attrs)*
        #[sea_orm(model_ex)]
        #vis struct #model {
            #(#model_fields),*
        }

        #(#model_ex_attrs)*
        #compact_model
        #vis struct #model_ex #all_fields
    })
}

pub fn expand_derive_model_ex(
    ident: Ident,
    data: Data,
    attrs: Vec<Attribute>,
) -> syn::Result<TokenStream> {
    let mut table_name = None;
    let mut compact = false;
    let mut model_fields: Vec<Ident> = Vec::new();
    let mut compound_fields: Vec<Ident> = Vec::new();
    let mut impl_related = Vec::new();
    let mut entity_loader_schema = EntityLoaderSchema::default();
    let mut unique_keys = BTreeMap::new();

    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("sea_orm"))
        .try_for_each(|attr| {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("table_name") {
                    table_name = Some(meta.value()?.parse::<LitStr>()?);
                } else if meta.path.is_ident("compact_model") {
                    compact = true;
                }
                Ok(())
            })
        })?;

    if let Data::Struct(item_struct) = &data {
        if let Fields::Named(fields) = &item_struct.fields {
            for field in &fields.named {
                if let Some(ident) = &field.ident {
                    let field_type = &field.ty;
                    let field_type = quote! { #field_type }
                        .to_string() // e.g.: "Option < String >"
                        .replace(' ', ""); // Remove spaces

                    if is_compound_field(&field_type) {
                        let compound_attrs =
                            compound_attr::SeaOrm::from_attributes(&field.attrs).ok();
                        if field_type.starts_with("HasOne<") {
                            entity_loader_schema.fields.push(EntityLoaderField {
                                is_one: true,
                                is_self: field_type == "HasOne<Entity>",
                                field: ident.clone(),
                                entity: extract_compound_entity(&field_type).to_owned(),
                                relation_enum: compound_attrs
                                    .as_ref()
                                    .map(|r| r.relation_enum.clone())
                                    .flatten(),
                            });
                        } else if field_type.starts_with("HasMany<") {
                            entity_loader_schema.fields.push(EntityLoaderField {
                                is_one: false,
                                is_self: field_type == "HasMany<Entity>",
                                field: ident.clone(),
                                entity: extract_compound_entity(&field_type).to_owned(),
                                relation_enum: compound_attrs
                                    .as_ref()
                                    .map(|r| r.relation_enum.clone())
                                    .flatten(),
                            });
                        }
                        if let Some(attrs) = compound_attrs {
                            if compact {
                                return Err(syn::Error::new_spanned(
                                    ident,
                                    "You cannot use #[has_one / has_many / belongs_to] on #[sea_orm::compact_model], please use #[sea_orm::model] instead.",
                                ));
                            }
                            impl_related.push((attrs, field_type));
                        }
                        compound_fields.push(format_field_ident_ref(field));
                    } else {
                        if let Ok(attrs) = column_attr::SeaOrm::from_attributes(&field.attrs) {
                            if attrs.unique.is_some() {
                                unique_keys
                                    .insert(ident.clone(), vec![(ident.clone(), field.ty.clone())]);
                            }
                            if let Some(unique_key) = attrs.unique_key {
                                unique_keys
                                    .entry(unique_key.parse()?)
                                    .or_default()
                                    .push((ident.clone(), field.ty.clone()));
                            }
                        }
                        model_fields.push(format_field_ident_ref(field));
                    }
                }
            }
        }
    }

    let impl_model_trait = DeriveModel::new(&ident, &data, &attrs)
        .map_err(|err| err.unwrap())?
        .impl_model_trait();

    let impl_from_model = quote! {
        impl Model {
            #[doc = " Generated by sea-orm-macros"]
            pub fn into_ex(self) -> ModelEx {
                self.into()
            }
        }

        #[automatically_derived]
        impl std::convert::From<Model> for ModelEx {
            fn from(m: Model) -> Self {
                Self {
                    #(#model_fields: m.#model_fields,)*
                    #(#compound_fields: Default::default(),)*
                }
            }
        }

        #[automatically_derived]
        impl std::convert::From<ModelEx> for Model {
            fn from(m: ModelEx) -> Self {
                Self {
                    #(#model_fields: m.#model_fields,)*
                }
            }
        }

        #[automatically_derived]
        impl PartialEq<ModelEx> for Model {
            fn eq(&self, other: &ModelEx) -> bool {
                true #(&& self.#model_fields == other.#model_fields)*
            }
        }

        #[automatically_derived]
        impl PartialEq<Model> for ModelEx {
            fn eq(&self, other: &Model) -> bool {
                true #(&& self.#model_fields == other.#model_fields)*
            }
        }
    };

    let mut relation_enum_variants: Punctuated<_, Comma> = Punctuated::new();
    let mut related_entity_enum_variants: Punctuated<_, Comma> = Punctuated::new();

    let impl_related_trait = {
        let mut ts = TokenStream::new();
        let mut seen = HashMap::new();
        for (_, field_type) in impl_related.iter() {
            *seen.entry(field_type).or_insert(0) += 1;
        }

        for (attrs, field_type) in impl_related.iter() {
            if attrs.self_ref.is_some() && attrs.relation_enum.is_none() {
                return Err(syn::Error::new_spanned(
                    ident,
                    "Please specify `relation_enum` for `self_ref`",
                ));
            }
            if let Some(var) = relation_enum_variant(attrs, field_type) {
                relation_enum_variants.push(var);
            }
            let (first, second) = related_entity_enum_variant(attrs, field_type);
            related_entity_enum_variants.push(first);
            if let Some(second) = second {
                related_entity_enum_variants.push(second);
            }
            if *seen.get(field_type).unwrap() == 1 {
                // prevent impl trait for same entity twice
                ts.extend(expand_impl_related_trait(attrs, field_type, &table_name)?);
            }
        }

        ts
    };

    let relation_enum = if !compact {
        quote! {
            #[doc = " Generated by sea-orm-macros"]
            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            pub enum Relation {
                #relation_enum_variants
            }
        }
    } else {
        // for backwards compatibility with compact models
        quote!()
    };

    let related_entity_enum = if !compact {
        quote! {
            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
            pub enum RelatedEntity {
                #related_entity_enum_variants
            }
        }
    } else {
        // for backwards compatibility with compact models
        quote!()
    };

    let (entity_find_by_key, loader_filter_by_key) = expand_find_by_unique_key(unique_keys);

    let entity_loader = expand_entity_loader(entity_loader_schema);

    Ok(quote! {
        #impl_from_model

        #impl_model_trait

        #relation_enum

        #impl_related_trait

        #related_entity_enum

        #entity_loader

        impl Entity {
            #entity_find_by_key
        }

        impl EntityLoader {
            #loader_filter_by_key
        }
    })
}

fn relation_enum_variant(attr: &compound_attr::SeaOrm, ty: &str) -> Option<TokenStream> {
    let (related_entity, relation_enum) = get_related(attr, ty);
    if attr.belongs_to.is_some() {
        let belongs_to = Ident::new("belongs_to", Span::call_site());

        let from = format_tuple(
            "",
            "Column",
            &attr
                .from
                .as_ref()
                .expect("Must specify `from` and `to` on belongs_to relation")
                .value(),
        );
        let to = format_tuple(
            related_entity.trim_end_matches("::Entity"),
            "Column",
            &attr
                .to
                .as_ref()
                .expect("Must specify `from` and `to` on belongs_to relation")
                .value(),
        );
        let mut extra: Punctuated<_, Comma> = Punctuated::new();
        if let Some(on_update) = &attr.on_update {
            let tag = Ident::new("on_update", on_update.span());
            extra.push(quote!(#tag = #on_update))
        }
        if let Some(on_delete) = &attr.on_delete {
            let tag = Ident::new("on_delete", on_delete.span());
            extra.push(quote!(#tag = #on_delete))
        }

        Some(quote! {
            #[sea_orm(#belongs_to = #related_entity, from = #from, to = #to, #extra)]
            #relation_enum
        })
    } else if attr.self_ref.is_some() {
        let belongs_to = Ident::new("belongs_to", Span::call_site());

        let from = format_tuple(
            "",
            "Column",
            &attr
                .from
                .as_ref()
                .expect("Must specify `from` and `to` on self_ref relation")
                .value(),
        );
        let to = format_tuple(
            "",
            "Column",
            &attr
                .to
                .as_ref()
                .expect("Must specify `from` and `to` on self_ref relation")
                .value(),
        );
        let mut extra: Punctuated<_, Comma> = Punctuated::new();
        if let Some(on_update) = &attr.on_update {
            let tag = Ident::new("on_update", on_update.span());
            extra.push(quote!(#tag = #on_update))
        }
        if let Some(on_delete) = &attr.on_delete {
            let tag = Ident::new("on_delete", on_delete.span());
            extra.push(quote!(#tag = #on_delete))
        }

        Some(quote! {
            #[sea_orm(#belongs_to = "Entity", from = #from, to = #to, #extra)]
            #relation_enum
        })
    } else if attr.has_many.is_some() && attr.via.is_none() {
        // skip junction relation

        let has_many = Ident::new("has_many", Span::call_site());
        let mut extra: Punctuated<_, Comma> = Punctuated::new();
        if let Some(via_rel) = &attr.via_rel {
            let tag = Ident::new("via_rel", via_rel.span());
            let via_rel = format!("Relation::{}", via_rel.value());
            extra.push(quote!(#tag = #via_rel))
        }

        Some(quote! {
            #[sea_orm(#has_many = #related_entity, #extra)]
            #relation_enum
        })
    } else if attr.has_one.is_some() {
        let has_one = Ident::new("has_one", Span::call_site());
        let mut extra: Punctuated<_, Comma> = Punctuated::new();
        if let Some(via_rel) = &attr.via_rel {
            let tag = Ident::new("via_rel", via_rel.span());
            let via_rel = format!("Relation::{}", via_rel.value());
            extra.push(quote!(#tag = #via_rel))
        }

        Some(quote! {
            #[sea_orm(#has_one = #related_entity, #extra)]
            #relation_enum
        })
    } else {
        None
    }
}

fn related_entity_enum_variant(
    attr: &compound_attr::SeaOrm,
    ty: &str,
) -> (TokenStream, Option<TokenStream>) {
    let (related_entity, relation_enum) = get_related(attr, ty);

    let extra = if attr.relation_enum.is_some() {
        let relation_def = format!("Relation::{relation_enum}.def()");
        quote!(, def = #relation_def)
    } else {
        quote!()
    };

    let first = quote! {
        #[sea_orm(entity = #related_entity #extra)]
        #relation_enum
    };
    let second = if attr.self_ref.is_some() {
        let relation_def = format!("Relation::{relation_enum}.def().rev()");
        let relation_enum_ref = if let Some(relation_reverse) = &attr.relation_reverse {
            Ident::new(&relation_reverse.value(), relation_reverse.span())
        } else {
            Ident::new(&format!("{relation_enum}Reverse"), relation_enum.span())
        };
        Some(quote! {
            #[sea_orm(entity = #related_entity def = #relation_def)]
            #relation_enum_ref
        })
    } else {
        None
    };

    (first, second)
}

fn expand_impl_related_trait(
    attr: &compound_attr::SeaOrm,
    ty: &str,
    table_name: &Option<LitStr>,
) -> syn::Result<TokenStream> {
    if attr.has_one.is_some() || attr.has_many.is_some() || attr.belongs_to.is_some() {
        let (related_entity, relation_enum) = get_related(attr, ty);
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
            let relation_def = quote!(Relation::#relation_enum.def());
            let via_relation_def: TokenStream =
                format!("Relation::{via_related}.def()").parse().unwrap();

            Ok(quote! {
                #[doc = " Generated by sea-orm-macros"]
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
            let relation_def = quote!(Relation::#relation_enum.def());

            Ok(quote! {
                #[doc = " Generated by sea-orm-macros"]
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

fn get_related<'a>(attr: &compound_attr::SeaOrm, ty: &'a str) -> (&'a str, Ident) {
    let related_entity = extract_compound_entity(ty);
    let relation_enum = if let Some(relation_enum) = &attr.relation_enum {
        Ident::new(
            &relation_enum.value().to_upper_camel_case(),
            relation_enum.span(),
        )
    } else {
        Ident::new(
            &infer_relation_name_from_entity(related_entity).to_upper_camel_case(),
            Span::call_site(),
        )
    };
    (related_entity, relation_enum)
}

fn extract_compound_entity(ty: &str) -> &str {
    if ty.starts_with("HasMany<") {
        &ty["HasMany<".len()..(ty.len() - 1)]
    } else if ty.starts_with("HasOne<") {
        &ty["HasOne<".len()..(ty.len() - 1)]
    } else if ty.starts_with("Option<") {
        &ty["Option<".len()..(ty.len() - 1)]
    } else if ty.starts_with("Vec<") {
        &ty["Vec<".len()..(ty.len() - 1)]
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

fn expand_find_by_unique_key(
    unique_keys: BTreeMap<Ident, Vec<(Ident, Type)>>,
) -> (TokenStream, TokenStream) {
    let mut entity_find_by_key = TokenStream::new();
    let mut loader_filter_by_key = TokenStream::new();

    for (name, columns) in unique_keys {
        let find_method = format_ident!("find_by_{}", name);
        let filter_method = format_ident!("filter_by_{}", name);
        if columns.len() > 1 {
            let key_type = columns.iter().map(|(_, ty)| ty).collect::<Vec<_>>();

            let filters = columns
                .iter()
                .enumerate()
                .map(|(i, (col, _))| {
                    let i = syn::Index::from(i);
                    let col = to_upper_camel_case(col);
                    quote!(Column::#col.eq(v.#i))
                })
                .collect::<Vec<_>>();

            entity_find_by_key.extend(quote! {
                #[doc = " Generated by sea-orm-macros"]
                pub fn #find_method(v: (#(#key_type),*)) -> Select<Entity> {
                    Self::find()
                        #(.filter(#filters))*
                }
            });
            loader_filter_by_key.extend(quote! {
                #[doc = " Generated by sea-orm-macros"]
                pub fn #filter_method(mut self, v: (#(#key_type),*)) -> Self {
                    #(self.filter_mut(#filters);)*
                    self
                }
            });
        } else {
            let col = to_upper_camel_case(&columns[0].0);
            let key_type = &columns[0].1;
            entity_find_by_key.extend(quote! {
                #[doc = " Generated by sea-orm-macros"]
                pub fn #find_method(v: impl Into<#key_type>) -> Select<Entity> {
                    Self::find().filter(Column::#col.eq(v.into()))
                }
            });
            loader_filter_by_key.extend(quote! {
                #[doc = " Generated by sea-orm-macros"]
                pub fn #filter_method(mut self, v: impl Into<#key_type>) -> Self {
                    self.filter_mut(Column::#col.eq(v.into()));
                    self
                }
            });
        }
    }

    (entity_find_by_key, loader_filter_by_key)
}

fn format_tuple(prefix: &str, middle: &str, suffix: &str) -> String {
    use std::fmt::Write;

    let parts = if suffix.starts_with('(') && suffix.ends_with(')') {
        suffix[1..suffix.len() - 1]
            .split(',')
            .map(|s| s.trim())
            .collect()
    } else {
        vec![suffix]
    };

    let mut output = String::new();
    if parts.len() > 1 {
        output.write_char('(').unwrap();
    }
    for (i, suffix) in parts.iter().enumerate() {
        let mut part = String::new();
        part.write_str(prefix).unwrap();
        if !part.is_empty() {
            part.write_str("::").unwrap();
        }
        part.write_str(middle).unwrap();
        part.write_str("::").unwrap();
        part.write_str(&suffix.to_upper_camel_case()).unwrap();

        if i > 0 {
            output.write_str(", ").unwrap();
        }
        output.write_str(&part).unwrap();
    }
    if parts.len() > 1 {
        output.write_char(')').unwrap();
    }

    output
}

fn to_upper_camel_case(i: &Ident) -> Ident {
    Ident::new(&i.to_string().to_upper_camel_case(), Span::call_site())
}

#[cfg(test)]
mod test {
    use super::format_tuple;

    #[test]
    fn test_format_tuple() {
        assert_eq!(format_tuple("", "Column", "Id"), "Column::Id");
        assert_eq!(format_tuple("super", "Column", "Id"), "super::Column::Id");
        assert_eq!(
            format_tuple("", "Column", "(A, B)"),
            "(Column::A, Column::B)"
        );
        assert_eq!(
            format_tuple("super", "Column", "(A, B)"),
            "(super::Column::A, super::Column::B)"
        );
    }
}
