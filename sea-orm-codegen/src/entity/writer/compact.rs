use super::*;

impl EntityWriter {
    #[allow(clippy::too_many_arguments)]
    pub fn gen_compact_code_blocks(
        entity: &Entity,
        with_serde: &WithSerde,
        column_option: &ColumnOption,
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
        let active_enums = Self::gen_import_active_enum(entity);
        imports.extend(active_enums.imports);
        let mut code_blocks = vec![
            imports,
            Self::gen_compact_model_struct(
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
            Self::gen_compact_relation_enum(entity),
        ];
        code_blocks.extend(Self::gen_impl_related(entity));
        code_blocks.extend(Self::gen_impl_conjunct_related(entity));
        if impl_active_model_behavior {
            code_blocks.extend([Self::impl_active_model_behavior()]);
        }
        if seaography {
            code_blocks.extend([Self::gen_related_entity(entity)]);
        }
        code_blocks
    }

    #[allow(clippy::too_many_arguments)]
    pub fn gen_compact_model_struct(
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

        quote! {
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
            }
        }
    }

    pub fn gen_compact_relation_enum(entity: &Entity) -> TokenStream {
        let attrs = entity.get_relation_attrs();
        let relation_enum_name = entity.get_relation_enum_name();
        quote! {
            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            pub enum Relation {
                #(
                    #attrs
                    #relation_enum_name,
                )*
            }
        }
    }
}
