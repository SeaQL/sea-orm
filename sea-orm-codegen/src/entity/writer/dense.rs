use super::*;
use crate::{Relation, RelationType};

impl EntityWriter {
    #[allow(clippy::too_many_arguments)]
    pub fn gen_dense_code_blocks(
        entity: &Entity,
        with_serde: &WithSerde,
        date_time_crate: &DateTimeCrate,
        schema_name: &Option<String>,
        serde_skip_deserializing_primary_key: bool,
        serde_skip_hidden_column: bool,
        model_extra_derives: &TokenStream,
        model_extra_attributes: &TokenStream,
        _column_extra_derives: &TokenStream,
        seaography: bool,
        impl_active_model_behavior: bool,
    ) -> Vec<TokenStream> {
        let mut imports = Self::gen_import(with_serde);
        imports.extend(Self::gen_import_active_enum(entity));
        let mut code_blocks = vec![
            imports,
            Self::gen_dense_model_struct(
                entity,
                with_serde,
                date_time_crate,
                schema_name,
                serde_skip_deserializing_primary_key,
                serde_skip_hidden_column,
                model_extra_derives,
                model_extra_attributes,
            ),
        ];
        if entity.relations.is_empty() && entity.conjunct_relations.is_empty() {
            code_blocks.push(Self::gen_compact_relation_enum(entity));
        }
        if impl_active_model_behavior {
            code_blocks.push(Self::impl_active_model_behavior());
        }
        if seaography {
            code_blocks.push(Self::gen_related_entity(entity));
        }
        code_blocks
    }

    #[allow(clippy::too_many_arguments)]
    pub fn gen_dense_model_struct(
        entity: &Entity,
        with_serde: &WithSerde,
        date_time_crate: &DateTimeCrate,
        schema_name: &Option<String>,
        serde_skip_deserializing_primary_key: bool,
        serde_skip_hidden_column: bool,
        model_extra_derives: &TokenStream,
        model_extra_attributes: &TokenStream,
    ) -> TokenStream {
        let table_name = entity.table_name.as_str();
        let column_names_snake_case = entity.get_column_names_snake_case();
        let column_rs_types = entity.get_column_rs_types(date_time_crate);
        let if_eq_needed = entity.get_eq_needed();
        let primary_keys: Vec<String> = entity
            .primary_keys
            .iter()
            .map(|pk| pk.name.clone())
            .collect();
        let attrs: Vec<TokenStream> = entity
            .columns
            .iter()
            .map(|col| {
                let mut attrs: Punctuated<_, Comma> = Punctuated::new();
                let is_primary_key = primary_keys.contains(&col.name);
                if !col.is_snake_case_name() {
                    let column_name = &col.name;
                    attrs.push(quote! { column_name = #column_name });
                }
                if is_primary_key {
                    attrs.push(quote! { primary_key });
                    if !col.auto_increment {
                        attrs.push(quote! { auto_increment = false });
                    }
                }
                if let Some(ts) = col.get_col_type_attrs() {
                    attrs.extend([ts]);
                    if !col.not_null {
                        attrs.push(quote! { nullable });
                    }
                };
                if col.unique {
                    attrs.push(quote! { unique });
                }
                let mut ts = quote! {};
                if !attrs.is_empty() {
                    for (i, attr) in attrs.into_iter().enumerate() {
                        if i > 0 {
                            ts = quote! { #ts, };
                        }
                        ts = quote! { #ts #attr };
                    }
                    ts = quote! { #[sea_orm(#ts)] };
                }
                let serde_attribute = col.get_serde_attribute(
                    is_primary_key,
                    serde_skip_deserializing_primary_key,
                    serde_skip_hidden_column,
                );
                ts = quote! {
                    #ts
                    #serde_attribute
                };
                ts
            })
            .collect();
        let schema_name = match Self::gen_schema_name(schema_name) {
            Some(schema_name) => quote! {
                schema_name = #schema_name,
            },
            None => quote! {},
        };
        let extra_derive = with_serde.extra_derive();

        let mut compound_objects: Punctuated<_, Comma> = Punctuated::new();

        let via_entities = entity.get_conjunct_relations_via_snake_case();
        for rel in entity.relations.iter() {
            if !rel.self_referencing && rel.num_suffix == 0 && rel.impl_related {
                let (rel_type, sea_orm_attr) = match rel.rel_type {
                    RelationType::HasOne => (format_ident!("HasOne"), quote!(#[sea_orm(has_one)])),
                    RelationType::HasMany => (format_ident!("Vec"), quote!(#[sea_orm(has_many)])),
                    RelationType::BelongsTo => {
                        let map_punctuated = |punctuated: Vec<String>| {
                            let len = punctuated.len();
                            let punctuated = punctuated.join(", ");
                            match len {
                                0..=1 => punctuated,
                                _ => format!("({punctuated})"),
                            }
                        };
                        let (from, to) = rel.get_src_ref_columns(
                            |x| x.to_string(),
                            |x| x.to_string(),
                            map_punctuated,
                        );
                        let on_update = if let Some(action) = &rel.on_update {
                            let action = Relation::get_foreign_key_action(action);
                            quote!(, on_update = #action)
                        } else {
                            quote!()
                        };
                        let on_delete = if let Some(action) = &rel.on_delete {
                            let action = Relation::get_foreign_key_action(action);
                            quote!(, on_delete = #action)
                        } else {
                            quote!()
                        };
                        (
                            format_ident!("HasOne"),
                            quote!(#[sea_orm(belongs_to, from = #from, to = #to #on_update #on_delete)]),
                        )
                    }
                };

                if let Some(to_entity) = rel.get_module_name() {
                    if !via_entities.contains(&to_entity) {
                        let field = if matches!(rel.rel_type, RelationType::HasMany) {
                            format_ident!(
                                "{}",
                                pluralizer::pluralize(&to_entity.to_string(), 2, false)
                            )
                        } else {
                            to_entity.clone()
                        };
                        compound_objects.push(quote! {
                            #sea_orm_attr
                            pub #field: #rel_type<super::#to_entity::Entity>
                        });
                    }
                }
            }
        }
        for (to_entity, via_entity) in entity
            .get_conjunct_relations_to_snake_case()
            .into_iter()
            .zip(via_entities)
        {
            let field = format_ident!(
                "{}",
                pluralizer::pluralize(&to_entity.to_string(), 2, false)
            );
            let via_entity = via_entity.to_string();
            compound_objects.push(quote! {
                #[sea_orm(has_many, via = #via_entity)]
                pub #field: Vec<super::#to_entity::Entity>
            });
        }

        if !compound_objects.is_empty() {
            compound_objects.push_punct(<syn::Token![,]>::default());
        }

        quote! {
            #[sea_orm::model]
            #[derive(Clone, Debug, PartialEq, DeriveEntityModel #if_eq_needed #extra_derive #model_extra_derives)]
            #[sea_orm(
                #schema_name
                table_name = #table_name
            )]
            #model_extra_attributes
            pub struct Model {
                #(
                    #attrs
                    pub #column_names_snake_case: #column_rs_types,
                )*
                #compound_objects
            }
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    #[ignore]
    fn test_name() {
        panic!("{}", pluralizer::pluralize("filling", 2, false));
    }
}
