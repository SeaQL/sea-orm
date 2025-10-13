use super::*;

impl EntityWriter {
    #[allow(clippy::too_many_arguments)]
    pub fn gen_expanded_code_blocks(
        entity: &Entity,
        with_serde: &WithSerde,
        date_time_crate: &DateTimeCrate,
        schema_name: &Option<String>,
        serde_skip_deserializing_primary_key: bool,
        serde_skip_hidden_column: bool,
        model_extra_derives: &TokenStream,
        model_extra_attributes: &TokenStream,
        column_extra_derives: &TokenStream,
        seaography: bool,
        impl_active_model_behavior: bool,
    ) -> Vec<TokenStream> {
        let mut imports = Self::gen_import(with_serde);
        imports.extend(Self::gen_import_active_enum(entity));
        let mut code_blocks = vec![
            imports,
            Self::gen_entity_struct(),
            Self::gen_impl_entity_name(entity, schema_name),
            Self::gen_expanded_model_struct(
                entity,
                with_serde,
                date_time_crate,
                serde_skip_deserializing_primary_key,
                serde_skip_hidden_column,
                model_extra_derives,
                model_extra_attributes,
            ),
            Self::gen_column_enum(entity, column_extra_derives),
            Self::gen_primary_key_enum(entity),
            Self::gen_impl_primary_key(entity, date_time_crate),
            Self::gen_relation_enum(entity),
            Self::gen_impl_column_trait(entity),
            Self::gen_impl_relation_trait(entity),
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

    pub fn gen_expanded_model_struct(
        entity: &Entity,
        with_serde: &WithSerde,
        date_time_crate: &DateTimeCrate,
        serde_skip_deserializing_primary_key: bool,
        serde_skip_hidden_column: bool,
        model_extra_derives: &TokenStream,
        model_extra_attributes: &TokenStream,
    ) -> TokenStream {
        let column_names_snake_case = entity.get_column_names_snake_case();
        let column_rs_types = entity.get_column_rs_types(date_time_crate);
        let if_eq_needed = entity.get_eq_needed();
        let serde_attributes = entity.get_column_serde_attributes(
            serde_skip_deserializing_primary_key,
            serde_skip_hidden_column,
        );
        let extra_derive = with_serde.extra_derive();

        quote! {
            #[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel #if_eq_needed #extra_derive #model_extra_derives)]
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
