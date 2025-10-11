use super::*;

impl EntityWriter {
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
        todo!()
    }
}
