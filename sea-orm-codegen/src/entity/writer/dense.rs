use super::*;
use crate::{Entity, Relation, RelationType};
use heck::ToSnakeCase;
use sea_query::ForeignKeyAction;

fn relation_column_name(a: &syn::Ident) -> String {
    let a = a.to_string();
    let b = a.to_snake_case();
    if a != b.to_upper_camel_case() {
        // if roundtrip fails, use original
        a
    } else {
        b
    }
}

fn relation_column_list(punctuated: Vec<String>) -> String {
    let len = punctuated.len();
    let punctuated = punctuated.join(", ");
    match len {
        0..=1 => punctuated,
        _ => format!("({punctuated})"),
    }
}

fn foreign_key_action_attr(
    attr: TokenStream,
    action: Option<&ForeignKeyAction>,
) -> Option<TokenStream> {
    action.map(|action| {
        let action = Relation::get_foreign_key_action(action);
        quote!(, #attr = #action)
    })
}

struct DenseRelationField<'a> {
    entity: &'a Entity,
    rel: &'a Relation,
    via_entities: &'a [syn::Ident],
}

impl DenseRelationField<'_> {
    fn field_tokens(&self) -> Option<TokenStream> {
        let (field, target_entity) = if self.rel.self_referencing {
            let table_name = self.entity.get_table_name_snake_case_ident();
            let suffix = if self.rel.num_suffix > 0 {
                format!("_{}", self.rel.num_suffix)
            } else {
                String::new()
            };
            let field = format_ident!("{table_name}{suffix}");

            (field, quote!(Entity))
        } else {
            if !self.rel.impl_related {
                return None;
            }

            let to_entity = self.rel.get_module_name()?;
            if self.via_entities.contains(&to_entity) {
                return None;
            }

            let field = match self.rel.rel_type {
                RelationType::HasMany => {
                    let to_entity = to_entity.to_string();
                    let pluralized = pluralizer::pluralize(&to_entity, 2, false);
                    format_ident!("{pluralized}")
                }
                RelationType::HasOne | RelationType::BelongsTo => to_entity.clone(),
            };
            let field = if self.rel.num_suffix == 0 {
                field
            } else {
                format_ident!("{field}_{}", self.rel.num_suffix)
            };
            (field, quote!(super::#to_entity::Entity))
        };

        let rel_field_type = match self.rel.rel_type {
            RelationType::BelongsTo => {
                let is_optional = !self.rel.columns.is_empty()
                    && self.rel.columns.iter().all(|name| {
                        self.entity
                            .columns
                            .iter()
                            .find(|column| column.name == *name)
                            .is_some_and(|column| column.not_null)
                    });
                if is_optional {
                    quote!(BelongsTo<#target_entity>)
                } else {
                    quote!(BelongsTo<Option<#target_entity>>)
                }
            }
            RelationType::HasOne => quote!(HasOne<#target_entity>),
            RelationType::HasMany => quote!(HasMany<#target_entity>),
        };
        let sea_orm_attr = if self.rel.self_referencing {
            let (from, to) = self.rel.get_src_ref_columns(
                relation_column_name,
                relation_column_name,
                relation_column_list,
            );
            let on_update = foreign_key_action_attr(quote!(on_update), self.rel.on_update.as_ref());
            let on_delete = foreign_key_action_attr(quote!(on_delete), self.rel.on_delete.as_ref());
            let relation_enum = self.rel.get_enum_name().to_string();

            quote!(#[sea_orm(self_ref, relation_enum = #relation_enum, from = #from, to = #to #on_update #on_delete)])
        } else {
            match self.rel.rel_type {
                RelationType::HasOne => quote!(#[sea_orm(has_one)]),
                RelationType::HasMany => quote!(#[sea_orm(has_many)]),
                RelationType::BelongsTo => {
                    let (from, to) = self.rel.get_src_ref_columns(
                        relation_column_name,
                        relation_column_name,
                        relation_column_list,
                    );
                    let on_update =
                        foreign_key_action_attr(quote!(on_update), self.rel.on_update.as_ref());
                    let on_delete =
                        foreign_key_action_attr(quote!(on_delete), self.rel.on_delete.as_ref());
                    let relation_enum = if self.rel.num_suffix > 0 {
                        let relation_enum = self.rel.get_enum_name().to_string();
                        Some(quote!(relation_enum = #relation_enum,))
                    } else {
                        None
                    };

                    quote!(#[sea_orm(belongs_to, #relation_enum from = #from, to = #to #on_update #on_delete)])
                }
            }
        };

        Some(quote! {
            #sea_orm_attr
            pub #field: #rel_field_type
        })
    }
}

impl EntityWriter {
    #[allow(clippy::too_many_arguments)]
    pub fn gen_dense_code_blocks(
        entity: &Entity,
        with_serde: &WithSerde,
        column_option: &ColumnOption,
        schema_name: &Option<String>,
        serde_skip_deserializing_primary_key: bool,
        serde_skip_hidden_column: bool,
        model_extra_derives: &TokenStream,
        model_extra_attributes: &TokenStream,
        _column_extra_derives: &TokenStream,
        _seaography: bool,
        impl_active_model_behavior: bool,
    ) -> Vec<TokenStream> {
        let mut imports = Self::gen_import(with_serde);
        let active_enums = Self::gen_import_active_enum(entity);
        imports.extend(active_enums.imports);
        let mut code_blocks = vec![
            imports,
            Self::gen_dense_model_struct(
                entity,
                with_serde,
                column_option,
                schema_name,
                serde_skip_deserializing_primary_key,
                serde_skip_hidden_column,
                model_extra_derives,
                model_extra_attributes,
                &active_enums.type_idents,
            ),
        ];
        if impl_active_model_behavior {
            code_blocks.push(Self::impl_active_model_behavior());
        }
        code_blocks
    }

    #[allow(clippy::too_many_arguments)]
    pub fn gen_dense_model_struct(
        entity: &Entity,
        with_serde: &WithSerde,
        column_option: &ColumnOption,
        schema_name: &Option<String>,
        serde_skip_deserializing_primary_key: bool,
        serde_skip_hidden_column: bool,
        model_extra_derives: &TokenStream,
        model_extra_attributes: &TokenStream,
        active_enum_type_idents: &ActiveEnumTypeIdents,
    ) -> TokenStream {
        let table_name = entity.table_name.as_str();
        let column_names_snake_case = entity.get_column_names_snake_case();
        let column_rs_types = Self::get_column_rs_types_with_enum_idents(
            entity,
            column_option,
            active_enum_type_idents,
        );
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
                } else if let Some(unique_key) = &col.unique_key {
                    attrs.push(quote! { unique_key = #unique_key });
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
            let relation_field = DenseRelationField {
                entity,
                rel,
                via_entities: &via_entities,
            };
            if let Some(field) = relation_field.field_tokens() {
                compound_objects.push(field);
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
                pub #field: HasMany<super::#to_entity::Entity>
            });
        }

        if !compound_objects.is_empty() {
            compound_objects.push_punct(<syn::Token![,]>::default());
        }

        quote! {
            #[sea_orm::model]
            #[derive(Clone, Debug, PartialEq #if_eq_needed, DeriveEntityModel #extra_derive #model_extra_derives)]
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

    #[allow(dead_code)]
    fn gen_dense_related_entity(entity: &Entity) -> TokenStream {
        let via_entities = entity.get_conjunct_relations_via_snake_case();

        let related_modules = entity.get_related_entity_modules();
        let related_attrs = entity.get_related_entity_attrs();
        let related_enum_names = entity.get_related_entity_enum_name();

        let items: Vec<_> = related_modules
            .into_iter()
            .zip(related_attrs)
            .zip(related_enum_names)
            .filter_map(|((related_module, related_attr), related_enum_name)| {
                if !via_entities.contains(&related_module) {
                    // skip junctions
                    Some(quote!(#related_attr #related_enum_name))
                } else {
                    None
                }
            })
            .collect();

        quote! {
            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
            pub enum RelatedEntity {
                #(#items),*
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
