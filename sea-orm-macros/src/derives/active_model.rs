use super::case_style::CaseStyle;
use super::util::{
    consume_meta, escape_rust_keyword, field_not_ignored, trim_starting_raw_identifier,
};
use heck::ToUpperCamelCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::str::FromStr;
use syn::{Attribute, Data, DataStruct, Expr, Fields, LitStr, Type, Visibility};

pub(crate) struct DeriveActiveModel {
    model: Ident,
    vis: Visibility,
    fields: Vec<Ident>,
    names: Vec<Ident>,
    types: Vec<Type>,
    #[cfg(feature = "with-json")]
    serde_serialize_names: Vec<String>,
}

impl DeriveActiveModel {
    pub fn new(
        vis: &Visibility,
        ident: &Ident,
        data: &Data,
        attrs: &[Attribute],
    ) -> syn::Result<Self> {
        let all_fields = match data {
            Data::Struct(DataStruct {
                fields: Fields::Named(named),
                ..
            }) => &named.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    ident,
                    "You can only derive DeriveActiveModel on structs",
                ));
            }
        };

        // Parse `#[serde(rename_all = "...")]` / `#[serde(rename_all(serialize = "...",
        // deserialize = "..."))]` at struct level, keeping only the serialize
        // side since these names are used to look up dummy values serialized
        // via `serde_json::to_value`.
        let mut serde_rename_all_serialize: Option<CaseStyle> = None;
        #[cfg(feature = "with-json")]
        attrs
            .iter()
            .filter(|attr| attr.path().is_ident("serde"))
            .try_for_each(|attr| {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("rename_all") {
                        if let Ok(lit) = meta.value().and_then(|v| v.parse::<LitStr>()) {
                            serde_rename_all_serialize = CaseStyle::from_str(&lit.value()).ok();
                        } else {
                            meta.parse_nested_meta(|nested| {
                                if nested.path.is_ident("serialize") {
                                    let lit: LitStr = nested.value()?.parse()?;
                                    serde_rename_all_serialize =
                                        CaseStyle::from_str(&lit.value()).ok();
                                } else {
                                    consume_meta(nested);
                                }
                                Ok(())
                            })?;
                        }
                    } else {
                        consume_meta(meta);
                    }
                    Ok(())
                })
            })?;

        let mut fields = Vec::new();
        let mut names = Vec::new();
        let mut types = Vec::new();
        #[cfg(feature = "with-json")]
        let mut serde_serialize_names = Vec::new();

        for field in all_fields.iter().filter(|f| field_not_ignored(f)) {
            let field_ident = field.ident.as_ref().expect("named fields have identifiers");
            let original_field_name = field_ident.to_string();
            let original_field_name = trim_starting_raw_identifier(original_field_name);
            fields.push(field_ident.clone());

            // Parse `#[serde(rename = "...")]` / `#[serde(rename(serialize = "..."))]`
            // at field level, keeping only the serialize side.
            let mut serde_rename_serialize: Option<String> = None;
            #[cfg(feature = "with-json")]
            field
                .attrs
                .iter()
                .filter(|attr| attr.path().is_ident("serde"))
                .try_for_each(|attr| {
                    attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("rename") {
                            if let Ok(lit) = meta.value().and_then(|v| v.parse::<LitStr>()) {
                                serde_rename_serialize = Some(lit.value());
                            } else {
                                meta.parse_nested_meta(|nested| {
                                    if nested.path.is_ident("serialize") {
                                        let lit: LitStr = nested.value()?.parse()?;
                                        serde_rename_serialize = Some(lit.value());
                                    } else {
                                        consume_meta(nested);
                                    }
                                    Ok(())
                                })?;
                            }
                        } else {
                            consume_meta(meta);
                        }
                        Ok(())
                    })
                })?;

            #[cfg(feature = "with-json")]
            {
                let name = if let Some(rename) = serde_rename_serialize.as_deref() {
                    rename.to_string()
                } else if let Some(case_style) = serde_rename_all_serialize {
                    super::entity_model::convert_case_public(&original_field_name, case_style)
                } else {
                    original_field_name.clone()
                };
                serde_serialize_names.push(name);
            }

            let ident = original_field_name.to_upper_camel_case();
            let ident = escape_rust_keyword(ident);
            let mut ident = format_ident!("{}", &ident);
            field
                .attrs
                .iter()
                .filter(|attr| attr.path().is_ident("sea_orm"))
                .try_for_each(|attr| {
                    attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("enum_name") {
                            let litstr: LitStr = meta.value()?.parse()?;
                            ident = syn::parse_str(&litstr.value()).unwrap();
                        } else {
                            // Reads the value expression to advance the parse stream.
                            // Some parameters, such as `primary_key`, do not have any value,
                            // so ignoring an error occurred here.
                            let _: Option<Expr> = meta.value().and_then(|v| v.parse()).ok();
                        }

                        Ok(())
                    })
                })?;

            names.push(ident);
            types.push(field.ty.clone());
        }

        Ok(DeriveActiveModel {
            model: ident.clone(),
            vis: vis.clone(),
            fields,
            names,
            types,
            #[cfg(feature = "with-json")]
            serde_serialize_names,
        })
    }
}

impl DeriveActiveModel {
    fn define_active_model(&self) -> TokenStream {
        let vis = &self.vis;
        let fields = &self.fields;
        let types = &self.types;
        quote!(
            #[doc = " Generated by sea-orm-macros"]
            #[derive(Clone, Debug, PartialEq)]
            #vis struct ActiveModel {
                #(
                    #[doc = " Generated by sea-orm-macros"]
                    pub #fields: sea_orm::ActiveValue<#types>
                ),*
            }
        )
    }

    fn impl_active_model(&self) -> TokenStream {
        let mut ts = self.impl_active_model_convert();
        ts.extend(self.impl_active_model_trait());
        ts
    }

    fn impl_active_model_convert(&self) -> TokenStream {
        let model = &self.model;
        let fields = &self.fields;

        quote!(
            #[automatically_derived]
            impl std::default::Default for ActiveModel {
                fn default() -> Self {
                    <Self as sea_orm::ActiveModelBehavior>::new()
                }
            }

            #[automatically_derived]
            impl std::convert::From<#model> for ActiveModel {
                fn from(m: #model) -> Self {
                    Self {
                        #(#fields: sea_orm::ActiveValue::Unchanged(m.#fields)),*
                    }
                }
            }

            #[automatically_derived]
            impl sea_orm::IntoActiveModel<ActiveModel> for #model {
                fn into_active_model(self) -> ActiveModel {
                    self.into()
                }
            }
        )
    }

    fn impl_active_model_trait(&self) -> TokenStream {
        let fields = &self.fields;
        let model = &self.model;
        let methods = self.impl_active_model_trait_methods();

        // A derive-generated `from_json` replaces the trait-default
        // implementation. The default merges dummy values serialized with
        // `sea_value_to_json_value` (SQL literal form), which cannot be
        // deserialized back by `Model` for types whose SQL literal form
        // differs from their serde representation (e.g. time-crate date and
        // time values). Here the dummy model is serialized with serde
        // instead, so the model round-trip always succeeds.
        let from_json_method = {
            #[cfg(feature = "with-json")]
            {
                let names = &self.names;
                let serde_serialize_names = &self.serde_serialize_names;
                quote! {
                    fn from_json(json: sea_orm::JsonValue) -> Result<Self, sea_orm::DbErr>
                    where
                        Self: sea_orm::TryIntoModel<#model>,
                        #model: sea_orm::IntoActiveModel<Self>,
                        for<'de> #model: serde::de::Deserialize<'de> + serde::Serialize,
                    {
                        use sea_orm::{ColumnTrait, IntoActiveModel, Iterable, TryIntoModel};
                        let sea_orm::JsonValue::Object(mut input) = json else {
                            return Err(sea_orm::DbErr::Json(format!(
                                "invalid type: expected JSON object for {}",
                                <<Self as sea_orm::ActiveModelTrait>::Entity as sea_orm::IdenStatic>::as_str(&Default::default())
                            )));
                        };

                        let dummy_am = <Self as sea_orm::ActiveModelTrait>::default_values();
                        let len = <<Self::Entity as sea_orm::EntityTrait>::Column>::iter().len();
                        // Mark down which attribute exists in the JSON object
                        let mut json_keys = Vec::with_capacity(len);

                        let dummy_model: #model = dummy_am.try_into_model()
                            .map_err(|e| sea_orm::DbErr::Json(e.to_string()))?;
                        // Serialize the dummy model with serde. Keys are the
                        // columns' serialize names, so look each one up by that
                        // name and re-key it to the column's json key (the
                        // deserialize name) used by `from_json`.
                        let mut merged = match serde_json::to_value(&dummy_model)
                            .map_err(|e| sea_orm::DbErr::Json(e.to_string()))?
                        {
                            sea_orm::JsonValue::Object(map) => map,
                            _ => serde_json::Map::new(),
                        };

                        for col in <<Self::Entity as sea_orm::EntityTrait>::Column>::iter() {
                            let key = col.json_key();
                            let has_key = input.contains_key(key);
                            json_keys.push((col, has_key));
                            if !has_key {
                                match col {
                                    #(
                                    <Self::Entity as sea_orm::EntityTrait>::Column::#names => {
                                        if let Some(value) = merged.remove(#serde_serialize_names) {
                                            merged.insert(key.to_owned(), value);
                                        }
                                    },
                                    )*
                                    _ => {}
                                }
                            }
                        }

                        merged.append(&mut input);
                        let _ = input;

                        let json_value = serde_json::Value::Object(merged);

                        // Convert JSON object into ActiveModel via Model
                        let model: #model = serde_json::from_value(json_value)
                            .map_err(|e| sea_orm::DbErr::Json(e.to_string()))?;
                        let mut am: Self = model.into_active_model();

                        // Transform attributes that exist in the JSON object
                        // into ActiveValue::Set, otherwise ActiveValue::NotSet
                        for (col, json_key_exists) in json_keys {
                            match (json_key_exists, am.get(col)) {
                                (true, sea_orm::ActiveValue::Set(value) | sea_orm::ActiveValue::Unchanged(value)) => {
                                    am.set(col, value);
                                }
                                _ => {
                                    am.not_set(col);
                                }
                            }
                        }

                        Ok(am)
                    }
                }
            }
            #[cfg(not(feature = "with-json"))]
            quote! {}
        };

        quote! {
            #[automatically_derived]
            impl sea_orm::ActiveModelTrait for ActiveModel {
                type Entity = Entity;

                #methods

                #from_json_method

                fn default() -> Self {
                    Self {
                        #(#fields: sea_orm::ActiveValue::NotSet),*
                    }
                }
            }
        }
    }

    pub fn impl_active_model_trait_methods(&self) -> TokenStream {
        let fields = &self.fields;
        let names = &self.names;

        quote!(
            fn take(&mut self, c: <Self::Entity as sea_orm::EntityTrait>::Column) -> sea_orm::ActiveValue<sea_orm::Value> {
                match c {
                    #(<Self::Entity as sea_orm::EntityTrait>::Column::#names => {
                        let mut value = sea_orm::ActiveValue::NotSet;
                        std::mem::swap(&mut value, &mut self.#fields);
                        value.into_wrapped_value()
                    },)*
                    _ => sea_orm::ActiveValue::NotSet,
                }
            }

            fn get(&self, c: <Self::Entity as sea_orm::EntityTrait>::Column) -> sea_orm::ActiveValue<sea_orm::Value> {
                match c {
                    #(<Self::Entity as sea_orm::EntityTrait>::Column::#names => self.#fields.clone().into_wrapped_value(),)*
                    _ => sea_orm::ActiveValue::NotSet,
                }
            }

            fn set_if_not_equals(&mut self, c: <Self::Entity as sea_orm::EntityTrait>::Column, v: sea_orm::Value) {
                match c {
                    #(<Self::Entity as sea_orm::EntityTrait>::Column::#names => self.#fields.set_if_not_equals(v.unwrap()),)*
                    _ => (),
                }
            }

            fn try_set(&mut self, c: <Self::Entity as sea_orm::EntityTrait>::Column, v: sea_orm::Value) -> Result<(), sea_orm::DbErr> {
                match c {
                    #(<Self::Entity as sea_orm::EntityTrait>::Column::#names => self.#fields = sea_orm::ActiveValue::Set(sea_orm::sea_query::ValueType::try_from(v).map_err(|e| sea_orm::DbErr::Type(e.to_string()))?),)*
                    _ => return Err(sea_orm::DbErr::Type(format!("ActiveModel does not have this field: {:?}", sea_orm::ColumnTrait::as_column_ref(&c)))),
                }
                Ok(())
            }

            fn not_set(&mut self, c: <Self::Entity as sea_orm::EntityTrait>::Column) {
                match c {
                    #(<Self::Entity as sea_orm::EntityTrait>::Column::#names => self.#fields = sea_orm::ActiveValue::NotSet,)*
                    _ => (),
                }
            }

            fn is_not_set(&self, c: <Self::Entity as sea_orm::EntityTrait>::Column) -> bool {
                match c {
                    #(<Self::Entity as sea_orm::EntityTrait>::Column::#names => self.#fields.is_not_set(),)*
                    _ => panic!("This ActiveModel does not have this field"),
                }
            }

            fn reset(&mut self, c: <Self::Entity as sea_orm::EntityTrait>::Column) {
                match c {
                    #(<Self::Entity as sea_orm::EntityTrait>::Column::#names => self.#fields.reset(),)*
                    _ => panic!("This ActiveModel does not have this field"),
                }
            }

            fn default_values() -> Self {
                use sea_orm::value::{DefaultActiveValue, DefaultActiveValueNone, DefaultActiveValueNotSet};
                let mut default = <Self as sea_orm::ActiveModelTrait>::default();
                #(default.#fields = (&default.#fields).default_value();)*
                default
            }
        )
    }
}

fn derive_into_model(ident: &Ident, data: &Data) -> syn::Result<TokenStream> {
    let model_fields = match data {
        Data::Struct(DataStruct {
            fields: Fields::Named(named),
            ..
        }) => &named.named,
        _ => {
            return Err(syn::Error::new_spanned(
                ident,
                "You can only derive DeriveActiveModel on structs",
            ));
        }
    };

    let active_model_field: Vec<Ident> = model_fields
        .iter()
        .filter(|f| field_not_ignored(f))
        .map(|field| field.ident.clone().expect("named fields have identifiers"))
        .collect();

    let model_field: Vec<Ident> = model_fields
        .iter()
        .map(|field| field.ident.clone().expect("named fields have identifiers"))
        .collect();

    let ignore_attr: Vec<bool> = model_fields.iter().map(|f| !field_not_ignored(f)).collect();

    let model_field_value: Vec<TokenStream> = model_field
        .iter()
        .zip(ignore_attr)
        .map(|(field, ignore)| {
            if ignore {
                quote! {
                    Default::default()
                }
            } else {
                quote! {
                    a.#field.unwrap()
                }
            }
        })
        .collect();

    Ok(quote!(
        #[automatically_derived]
        impl std::convert::TryFrom<ActiveModel> for #ident {
            type Error = sea_orm::DbErr;
            fn try_from(a: ActiveModel) -> Result<Self, sea_orm::DbErr> {
                #(if a.#active_model_field.is_not_set() {
                    return Err(sea_orm::DbErr::AttrNotSet(stringify!(#active_model_field).to_owned()));
                })*
                Ok(
                    Self {
                        #(#model_field: #model_field_value),*
                    }
                )
            }
        }

        #[automatically_derived]
        impl sea_orm::TryIntoModel<#ident> for ActiveModel {
            fn try_into_model(self) -> Result<#ident, sea_orm::DbErr> {
                self.try_into()
            }
        }
    ))
}

pub fn expand_derive_active_model(
    vis: &Visibility,
    ident: &Ident,
    data: &Data,
    attrs: &[Attribute],
) -> syn::Result<TokenStream> {
    let derive_active_model = DeriveActiveModel::new(vis, ident, data, attrs)?;

    let define_active_model = derive_active_model.define_active_model();
    let impl_active_model = derive_active_model.impl_active_model();
    let derive_into_model = derive_into_model(ident, data)?;

    Ok(quote!(
        #define_active_model

        #impl_active_model

        #derive_into_model
    ))
}
