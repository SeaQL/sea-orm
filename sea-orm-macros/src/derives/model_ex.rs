use super::attributes::compound_attr;
use super::entity_loader::{EntityLoaderField, EntityLoaderSchema, expand_entity_loader};
use super::util::{extract_compound_entity, format_field_ident, is_compound_field};
use super::{expand_typed_column, model::DeriveModel};
use heck::ToUpperCamelCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use std::collections::{BTreeMap, HashMap};
use syn::{
    Attribute, Data, Expr, Fields, ItemStruct, Lit, Meta, Type, parse_quote,
    punctuated::Punctuated, token::Comma,
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
                        new_list.push(parse_quote!(DeriveActiveModelEx));
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
                if meta.path.is_ident("compact_model") {
                    compact = true;
                } else {
                    // Reads the value expression to advance the parse stream.
                    let _: Option<Expr> = meta.value().and_then(|v| v.parse()).ok();
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
                        let is_reverse = compound_attrs
                            .as_ref()
                            .map(|r| r.reverse.is_some())
                            .unwrap_or_default();
                        let relation_enum = compound_attrs
                            .as_ref()
                            .and_then(|r| r.relation_enum.clone());
                        if field_type.starts_with("HasOne<") {
                            entity_loader_schema.fields.push(EntityLoaderField {
                                is_one: true,
                                is_self: field_type == "HasOne<Entity>",
                                is_reverse,
                                field: ident.clone(),
                                entity: extract_compound_entity(&field_type).to_owned(),
                                relation_enum,
                                via: None,
                            });
                        } else if field_type.starts_with("HasMany<") {
                            entity_loader_schema.fields.push(EntityLoaderField {
                                is_one: false,
                                is_self: field_type == "HasMany<Entity>",
                                is_reverse,
                                field: ident.clone(),
                                entity: extract_compound_entity(&field_type).to_owned(),
                                relation_enum,
                                via: compound_attrs.as_ref().and_then(|r| r.via.clone()),
                            });
                        }
                        if let Some(attrs) = compound_attrs {
                            if compact
                                && (attrs.has_one.is_some()
                                    || attrs.has_many.is_some()
                                    || attrs.belongs_to.is_some())
                            {
                                return Err(syn::Error::new_spanned(
                                    ident,
                                    "You cannot use #[has_one / has_many / belongs_to] on #[sea_orm::compact_model], please use #[sea_orm::model] instead.",
                                ));
                            } else if attrs.belongs_to.is_some()
                                && !field_type.starts_with("HasOne<")
                            {
                                return Err(syn::Error::new_spanned(
                                    ident,
                                    "belongs_to must be paired with HasOne",
                                ));
                            } else if attrs.has_one.is_some() && !field_type.starts_with("HasOne<")
                            {
                                return Err(syn::Error::new_spanned(
                                    ident,
                                    "has_one must be paired with HasOne",
                                ));
                            } else if attrs.has_many.is_some()
                                && !field_type.starts_with("HasMany<")
                            {
                                return Err(syn::Error::new_spanned(
                                    ident,
                                    "has_many must be paired with HasMany",
                                ));
                            }
                            impl_related.push((attrs, field_type));
                        }
                        compound_fields.push(format_field_ident(field));
                    } else {
                        // scalar field
                        for attr in field.attrs.iter() {
                            // still have to parse column attributes to extract unique keys
                            if attr.path().is_ident("sea_orm") {
                                attr.parse_nested_meta(|meta| {
                                    if meta.path.is_ident("unique") {
                                        unique_keys.insert(
                                            ident.clone(),
                                            vec![(ident.clone(), field.ty.clone())],
                                        );
                                    } else if meta.path.is_ident("unique_key") {
                                        let lit = meta.value()?.parse()?;
                                        if let Lit::Str(litstr) = lit {
                                            unique_keys
                                                .entry(litstr.parse()?)
                                                .or_default()
                                                .push((ident.clone(), field.ty.clone()));
                                        } else {
                                            return Err(
                                                meta.error(format!("Invalid unique_key {lit:?}"))
                                            );
                                        }
                                    } else {
                                        // Reads the value expression to advance the parse stream.
                                        let _: Option<Expr> =
                                            meta.value().and_then(|v| v.parse()).ok();
                                    }

                                    Ok(())
                                })?;
                            }
                        }
                        model_fields.push(format_field_ident(field));
                    }
                }
            }
        }
    }

    let impl_model_trait = DeriveModel::new(&ident, &data, &attrs)?.impl_model_trait();

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
        let mut seen_entity = HashMap::new();
        for (_, field_type) in impl_related.iter() {
            let entity_path = extract_compound_entity(field_type);
            *seen_entity.entry(entity_path).or_insert(0) += 1;
        }

        for (attrs, field_type) in impl_related.iter() {
            if attrs.self_ref.is_some() && attrs.via.is_some() {
                ts.extend(expand_impl_related_self_via(attrs, field_type)?);
            } else {
                if attrs.self_ref.is_some() && attrs.relation_enum.is_none() {
                    return Err(syn::Error::new_spanned(
                        ident,
                        "Please specify `relation_enum` for `self_ref`",
                    ));
                }
                if let Some(var) = relation_enum_variant(attrs, field_type) {
                    relation_enum_variants.push(var);
                }
                if attrs.self_ref.is_some() && field_type.starts_with("HasMany<") {
                    // related entity is already provided by the HasOne item
                    // so self_ref HasMany has to be skipped
                    continue;
                }
                let (first, second) = related_entity_enum_variant(attrs, field_type);
                related_entity_enum_variants.push(first);
                if let Some(second) = second {
                    related_entity_enum_variants.push(second);
                }
                let entity_path = extract_compound_entity(field_type);
                if *seen_entity.get(entity_path).unwrap() == 1 {
                    // prevent impl trait for same entity twice
                    ts.extend(expand_impl_related_trait(attrs, field_type)?);
                }
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
            #[doc = " Generated by sea-orm-macros"]
            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
            pub enum RelatedEntity {
                #related_entity_enum_variants
            }
        }
    } else {
        // for backwards compatibility with compact models
        quote!()
    };

    let (typed_column, typed_column_const) = expand_typed_column(&data)?;

    let (entity_find_by_key, loader_filter_by_key) = expand_find_by_unique_key(unique_keys);

    let entity_loader = expand_entity_loader(entity_loader_schema);

    Ok(quote! {
        #typed_column

        #typed_column_const

        #impl_from_model

        #impl_model_trait

        #relation_enum

        #impl_related_trait

        #related_entity_enum

        #entity_loader

        impl Entity {
            #[doc = " Generated by sea-orm-macros"]
            pub const COLUMN: TypedColumn = COLUMN;

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
        if let Some(()) = &attr.skip_fk {
            extra.push(quote!(skip_fk))
        }

        Some(quote! {
            #[doc = " Generated by sea-orm-macros"]
            #[sea_orm(#belongs_to = #related_entity, from = #from, to = #to, #extra)]
            #relation_enum
        })
    } else if attr.self_ref.is_some()
        && attr.via.is_none()
        && attr.from.is_some()
        && attr.to.is_some()
    {
        let belongs_to = Ident::new("belongs_to", Span::call_site());

        let from = format_tuple("", "Column", &attr.from.as_ref().unwrap().value());
        let to = format_tuple("", "Column", &attr.to.as_ref().unwrap().value());
        let mut extra: Punctuated<_, Comma> = Punctuated::new();
        if let Some(on_update) = &attr.on_update {
            let tag = Ident::new("on_update", on_update.span());
            extra.push(quote!(#tag = #on_update))
        }
        if let Some(on_delete) = &attr.on_delete {
            let tag = Ident::new("on_delete", on_delete.span());
            extra.push(quote!(#tag = #on_delete))
        }
        if let Some(()) = &attr.skip_fk {
            extra.push(quote!(skip_fk))
        }

        Some(quote! {
            #[doc = " Generated by sea-orm-macros"]
            #[sea_orm(#belongs_to = "Entity", from = #from, to = #to, #extra)]
            #relation_enum
        })
    } else if attr.self_ref.is_some()
        && attr.via.is_none()
        && attr.relation_reverse.is_some()
        && ty.starts_with("HasMany<")
    {
        let has_many = Ident::new("has_many", Span::call_site());

        #[allow(clippy::unnecessary_unwrap)]
        let via_rel = format!(
            "Relation::{}",
            attr.relation_reverse.as_ref().unwrap().value()
        );

        Some(quote! {
            #[doc = " Generated by sea-orm-macros"]
            #[sea_orm(#has_many = "Entity", via_rel = #via_rel)]
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
            #[doc = " Generated by sea-orm-macros"]
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
            #[doc = " Generated by sea-orm-macros"]
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
        #[doc = " Generated by sea-orm-macros"]
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
            #[doc = " Generated by sea-orm-macros"]
            #[sea_orm(entity = #related_entity def = #relation_def)]
            #relation_enum_ref
        })
    } else {
        None
    };

    (first, second)
}

fn expand_impl_related_trait(attr: &compound_attr::SeaOrm, ty: &str) -> syn::Result<TokenStream> {
    if attr.has_one.is_some() || attr.has_many.is_some() || attr.belongs_to.is_some() {
        let (related_entity, relation_enum) = get_related(attr, ty);
        let related_entity: TokenStream = related_entity.parse().unwrap();

        if let Some(via_lit) = &attr.via {
            let via = via_lit.value();
            let mut junction = via.as_str();
            let mut via_related = "";
            if let Some((prefix, suffix)) = via.split_once("::") {
                junction = prefix;
                via_related = suffix;
            }
            let junction = Ident::new(junction, via_lit.span());
            let relation_def = quote!(super::#junction::Relation::#relation_enum.def());
            let via_relation_def: TokenStream = if !via_related.is_empty() {
                let via_related = Ident::new(via_related, via_lit.span());
                quote!(super::#junction::Relation::#via_related.def().rev())
            } else {
                quote!(<super::#junction::Entity as Related<Entity>>::to().rev())
            };

            Ok(quote! {
                #[doc = " Generated by sea-orm-macros"]
                impl Related<#related_entity> for Entity {
                    fn to() -> RelationDef {
                        #relation_def
                    }
                    fn via() -> Option<RelationDef> {
                        Some(#via_relation_def)
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

fn expand_impl_related_self_via(
    attr: &compound_attr::SeaOrm,
    ty: &str,
) -> syn::Result<TokenStream> {
    let Some(via) = &attr.via else {
        return Err(syn::Error::new(
            Span::call_site(),
            "Please specify the junction Entity `via` for `self_ref`.",
        ));
    };

    if ty != "HasMany<Entity>" {
        return Err(syn::Error::new_spanned(
            via,
            "self_ref + via field type must be `HasMany<Entity>`",
        ));
    }

    if attr.reverse.is_some() {
        return Ok(quote!());
    }

    if let (Some(from), Some(to)) = (&attr.from, &attr.to) {
        let junction = Ident::new(&via.value(), via.span());
        let from = Ident::new(&from.value(), from.span());
        let to = Ident::new(&to.value(), to.span());

        Ok(quote! {
            #[doc = " Generated by sea-orm-macros"]
            impl RelatedSelfVia<super::#junction::Entity> for Entity {
                fn to() -> RelationDef {
                    super::#junction::Relation::#to.def()
                }
                fn via() -> RelationDef {
                    super::#junction::Relation::#from.def().rev()
                }
            }
        })

        // #[sea_orm(self_ref, via = "user_follower", from = "User", to = "Follower")]
        // impl RelatedSelfVia<super::user_follower::Entity> for Entity {
        //     fn to() -> RelationDef {
        //         super::user_follower::Relation::Follower.def()
        //     }

        //     fn via() -> RelationDef {
        //         super::user_follower::Relation::User.def().rev()
        //     }
        // }
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
        let delete_method = format_ident!("delete_by_{}", name);
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

                #[doc = " Generated by sea-orm-macros"]
                pub fn #delete_method(v: (#(#key_type),*)) -> sea_orm::ValidatedDeleteOne<Entity> {
                    sea_orm::Delete::_one_only_for_use_by_model_ex(Entity)
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
