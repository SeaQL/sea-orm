use super::*;

impl EntityWriter {
    #[allow(clippy::too_many_arguments)]
    pub fn gen_expanded_code_blocks(
        entity: &Entity,
        with_serde: &WithSerde,
        column_option: &ColumnOption,
        schema_name: &Option<String>,
        serde_skip_deserializing_primary_key: bool,
        serde_skip_hidden_column: bool,
        model_extra_derives: &TokenStream,
        model_extra_attributes: &TokenStream,
        column_extra_derives: &TokenStream,
        seaography: bool,
        impl_active_model_behavior: bool,
        sea_orm_feature: &Option<String>,
    ) -> Vec<TokenStream> {
        let mut imports = Self::gen_import(with_serde, sea_orm_feature);
        imports.extend(Self::gen_import_active_enum(entity));
        let mut code_blocks = vec![
            imports,
            Self::gen_entity_struct(sea_orm_feature),
            Self::wrap_impl_feature_gate(
                Self::gen_impl_entity_name(entity, schema_name),
                sea_orm_feature,
            ),
            Self::gen_expanded_model_struct(
                entity,
                with_serde,
                column_option,
                serde_skip_deserializing_primary_key,
                serde_skip_hidden_column,
                model_extra_derives,
                model_extra_attributes,
                sea_orm_feature,
            ),
            Self::gen_column_enum(entity, column_extra_derives, sea_orm_feature),
            Self::gen_primary_key_enum(entity, sea_orm_feature),
            Self::wrap_impl_feature_gate(
                Self::gen_impl_primary_key(entity, column_option),
                sea_orm_feature,
            ),
            Self::gen_relation_enum(entity, sea_orm_feature),
            Self::wrap_impl_feature_gate(Self::gen_impl_column_trait(entity), sea_orm_feature),
            Self::wrap_impl_feature_gate(Self::gen_impl_relation_trait(entity), sea_orm_feature),
        ];

        code_blocks.extend(
            Self::gen_impl_related(entity)
                .into_iter()
                .map(|code| Self::wrap_impl_feature_gate(code, sea_orm_feature)),
        );
        code_blocks.extend(
            Self::gen_impl_conjunct_related(entity)
                .into_iter()
                .map(|code| Self::wrap_impl_feature_gate(code, sea_orm_feature)),
        );

        if impl_active_model_behavior {
            code_blocks.push(Self::wrap_impl_feature_gate(Self::impl_active_model_behavior(), sea_orm_feature));
        }
        if seaography {
            code_blocks.push(Self::gen_related_entity(entity, sea_orm_feature));
        }
        code_blocks
    }

    pub fn gen_expanded_model_struct(
        entity: &Entity,
        with_serde: &WithSerde,
        column_option: &ColumnOption,
        serde_skip_deserializing_primary_key: bool,
        serde_skip_hidden_column: bool,
        model_extra_derives: &TokenStream,
        model_extra_attributes: &TokenStream,
        sea_orm_feature: &Option<String>,
    ) -> TokenStream {
        let column_names_snake_case = entity.get_column_names_snake_case();
        let column_rs_types = entity.get_column_rs_types(column_option);
        let if_eq_needed = entity.get_eq_needed();
        let serde_attributes = entity.get_column_serde_attributes(
            serde_skip_deserializing_primary_key,
            serde_skip_hidden_column,
        );
        let extra_derive = with_serde.extra_derive();

        let (additional_derives, additional_attributes) = if let Some(feature) = sea_orm_feature {
            (
                quote! {},
                quote! { #[cfg_attr(feature = #feature, derive(DeriveModel, DeriveActiveModel))] },
            )
        } else {
            (quote! { , DeriveModel, DeriveActiveModel }, quote! {})
        };

        quote! {
            #[derive(Clone, Debug, PartialEq #additional_derives #if_eq_needed #extra_derive #model_extra_derives)]
            #additional_attributes
            #model_extra_attributes
            pub struct Model {
                #(
                    #serde_attributes
                    pub #column_names_snake_case: #column_rs_types,
                )*
            }
        }
    }
}
