#[cfg(feature = "with-json")]
use super::serde_attributes::SerdeMeta;
use super::util::{escape_rust_keyword, field_not_ignored, trim_starting_raw_identifier};
use heck::ToUpperCamelCase;
#[cfg(feature = "with-json")]
use proc_macro2::Span;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Data, DataStruct, DeriveInput, Expr, Fields, FieldsNamed, LitStr, Type, Visibility};

struct ActiveModelField<'a> {
    ident: &'a Ident,
    column: Ident,
    ty: &'a Type,
}

pub(super) struct ActiveModelFields<'a>(Vec<ActiveModelField<'a>>);

impl<'a> ActiveModelFields<'a> {
    pub(super) fn new(model_fields: &'a FieldsNamed) -> syn::Result<Self> {
        let mut fields = Vec::new();

        for field in model_fields.named.iter().filter(|f| field_not_ignored(f)) {
            let field_ident = field.ident.as_ref().expect("named fields have identifiers");

            let ident = field_ident.to_string();
            let ident = trim_starting_raw_identifier(ident).to_upper_camel_case();
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

            fields.push(ActiveModelField {
                ident: field_ident,
                column: ident,
                ty: &field.ty,
            });
        }

        Ok(Self(fields))
    }

    pub(super) fn impl_active_model_trait_methods(&self) -> TokenStream {
        let (fields, names): (Vec<_>, Vec<_>) = self
            .0
            .iter()
            .map(|field| (field.ident, &field.column))
            .unzip();

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

pub struct DeriveActiveModel<'a> {
    model: &'a Ident,
    vis: &'a Visibility,
    model_fields: &'a FieldsNamed,
    active_fields: ActiveModelFields<'a>,
    #[cfg(feature = "with-json")]
    pub(crate) serde_meta: SerdeMeta<'a>,
}

impl<'a> TryFrom<&'a DeriveInput> for DeriveActiveModel<'a> {
    type Error = syn::Error;

    fn try_from(input: &'a DeriveInput) -> syn::Result<Self> {
        let model_fields = match &input.data {
            Data::Struct(DataStruct {
                fields: Fields::Named(named),
                ..
            }) => named,
            _ => {
                return Err(syn::Error::new_spanned(
                    &input.ident,
                    "You can only derive DeriveActiveModel on structs",
                ));
            }
        };

        Ok(Self {
            model: &input.ident,
            vis: &input.vis,
            model_fields,
            active_fields: ActiveModelFields::new(model_fields)?,
            #[cfg(feature = "with-json")]
            serde_meta: SerdeMeta::new(input)?,
        })
    }
}

impl DeriveActiveModel<'_> {
    pub(crate) fn expand(&self) -> TokenStream {
        let vis = &self.vis;
        let fields = self.active_fields.0.iter().map(|field| field.ident);
        let types = self.active_fields.0.iter().map(|field| field.ty);
        let impl_active_model_convert = self.impl_active_model_convert();
        let impl_active_model_trait = self.impl_active_model_trait();
        let derive_into_model = self.derive_into_model();

        quote!(
            #[doc = " Generated by sea-orm-macros"]
            #[derive(Clone, Debug, PartialEq)]
            #vis struct ActiveModel {
                #(
                    #[doc = " Generated by sea-orm-macros"]
                    pub #fields: sea_orm::ActiveValue<#types>
                ),*
            }

            #impl_active_model_convert

            #impl_active_model_trait

            #derive_into_model
        )
    }

    fn impl_active_model_convert(&self) -> TokenStream {
        let model = &self.model;
        let fields = self.active_fields.0.iter().map(|field| field.ident);

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

    fn derive_into_model(&self) -> TokenStream {
        let ident = self.model;
        let model_fields = &self.model_fields.named;

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

        quote!(
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
        )
    }

    fn impl_active_model_trait(&self) -> TokenStream {
        let fields = self.active_fields.0.iter().map(|field| field.ident);
        let methods = self.active_fields.impl_active_model_trait_methods();
        #[cfg(feature = "with-json")]
        let from_json = self.impl_from_json();
        #[cfg(not(feature = "with-json"))]
        let from_json = TokenStream::new();

        quote! {
            #[automatically_derived]
            impl sea_orm::ActiveModelTrait for ActiveModel {
                type Entity = Entity;

                #methods

                #from_json

                fn default() -> Self {
                    Self {
                        #(#fields: sea_orm::ActiveValue::NotSet),*
                    }
                }
            }
        }
    }

    #[cfg(feature = "with-json")]
    fn impl_from_json(&self) -> TokenStream {
        let serde_meta = &self.serde_meta.container;

        if serde_meta.attrs.transparent()
            || serde_meta.attrs.type_from().is_some()
            || serde_meta.attrs.type_try_from().is_some()
        {
            return quote! {};
        }

        // Entity models need not implement Deserialize, so field bounds cannot be required by the generated trait method.
        let marker = Ident::new("SeaOrmFromJson", Span::mixed_site());
        let decoder = Ident::new("SeaOrmFromJsonDecoder", Span::mixed_site());
        let deserialize = Ident::new("sea_orm_from_json", Span::mixed_site());
        let input = Ident::new("SeaOrmJsonInput", Span::mixed_site());
        let deserialize_and_wrap = Ident::new("sea_orm_deserialize_and_wrap", Span::mixed_site());

        let mut deserialize_bound_types = Vec::new();
        let mut input_fields = Vec::new();
        let mut ignored_keys = Vec::new();
        let mut fn_defs = Vec::new();
        let mut active_model_values = Vec::new();

        for (index, field) in serde_meta.data.all_fields().enumerate() {
            let is_active_field = field_not_ignored(field.original);
            let ty = field.ty;
            let attrs = &field.attrs;
            let field = &field.member;

            if !is_active_field {
                if !attrs.flatten() && !attrs.skip_deserializing() {
                    let name = attrs.name().deserialize_name();
                    let aliases = attrs.aliases();

                    // These are known Model keys, but their values must not be parsed as database fields.
                    ignored_keys.push(name);
                    ignored_keys.extend(aliases.iter().map(String::as_str));
                }
                continue;
            }

            if attrs.flatten() || attrs.skip_deserializing() {
                active_model_values.push(quote! { #field: sea_orm::ActiveValue::NotSet, });
                continue;
            }

            let name = attrs.name().deserialize_name();
            let aliases = attrs.aliases();
            let deserialize_attr = if let Some(path) = attrs.deserialize_with() {
                let deserialize_field_fn =
                    format_ident!("sea_orm_deserialize_{index}", span = Span::mixed_site());

                fn_defs.push(quote! {
                    fn #deserialize_field_fn<'de, D>(deserializer: D) -> Result<sea_orm::ActiveValue<#ty>, D::Error>
                    where D: sea_orm::serde::Deserializer<'de>,
                    {
                        #path(deserializer).map(sea_orm::ActiveValue::Set)
                    }
                });

                let path = deserialize_field_fn.to_string();
                quote! { #[serde(deserialize_with = #path)] }
            } else {
                // Custom deserializers above provide their own parsing; only
                // regular deserialization requires the field's Deserialize bound.
                deserialize_bound_types.push(ty);

                let path = deserialize_and_wrap.to_string();
                quote! { #[serde(deserialize_with = #path)] }
            };

            let default_value = match attrs.default() {
                serde_derive_internals::attr::Default::None => None,
                serde_derive_internals::attr::Default::Default => {
                    Some(quote! { <#ty as std::default::Default>::default() })
                }
                serde_derive_internals::attr::Default::Path(path) => Some(quote! { #path() }),
            };
            let default_attr = if let Some(value) = default_value {
                let default_field_fn =
                    format_ident!("sea_orm_default_{index}", span = Span::mixed_site());

                fn_defs.push(quote! {
                    fn #default_field_fn() -> sea_orm::ActiveValue<#ty> {
                        sea_orm::ActiveValue::Set(#value)
                    }
                });
                let path = default_field_fn.to_string();
                quote! { #[serde(default = #path)] }
            } else {
                quote! { #[serde(default)] }
            };

            input_fields.push(quote! {
                #[serde(rename = #name, #(alias = #aliases,)*)]
                #default_attr
                #deserialize_attr
                #field: sea_orm::ActiveValue<#ty>,
            });
            active_model_values.push(quote! {
                #field: input.#field,
            });
        }

        let bounds = quote! {
            #(#deserialize_bound_types: sea_orm::serde::Deserialize<'de>,)*
        }
        .to_string();

        if !deserialize_bound_types.is_empty() {
            fn_defs.push(quote! {
                fn #deserialize_and_wrap<'de, D, T>(deserializer: D) -> Result<sea_orm::ActiveValue<T>, D::Error>
                where
                    D: sea_orm::serde::Deserializer<'de>,
                    T: sea_orm::serde::Deserialize<'de> + Into<sea_orm::Value>,
                {
                    T::deserialize(deserializer).map(sea_orm::ActiveValue::Set)
                }
            });
        }

        let unknown_field = if serde_meta.attrs.deny_unknown_fields() {
            quote! { #[serde(deny_unknown_fields)] }
        } else {
            quote! {}
        };
        let object_binding = if ignored_keys.is_empty() {
            quote! { object }
        } else {
            quote! { mut object }
        };

        quote! {
            fn from_json(json: sea_orm::serde_json::Value) -> Result<Self, sea_orm::DbErr>
            where
                Self: sea_orm::TryIntoModel<<Self::Entity as sea_orm::EntityTrait>::Model>,
                <Self::Entity as sea_orm::EntityTrait>::Model: sea_orm::IntoActiveModel<Self>,
                for<'de> <Self::Entity as sea_orm::EntityTrait>::Model:
                    sea_orm::serde::de::Deserialize<'de> + sea_orm::serde::Serialize,
            {
                let sea_orm::serde_json::Value::Object(#object_binding) = json else {
                    return Err(sea_orm::DbErr::Json(format!(
                        "invalid type: expected JSON object for {}",
                        sea_orm::IdenStatic::as_str(&Entity::default()),
                    )));
                };
                #(object.remove(#ignored_keys);)*

                #(#fn_defs)*

                #[derive(sea_orm::serde::Deserialize)]
                #[serde(crate = "sea_orm::serde", bound(deserialize = #bounds))]
                #unknown_field
                struct #input {
                    #(#input_fields)*
                }

                impl From<#input> for ActiveModel {
                    fn from(input: #input) -> Self {
                        Self {
                            #(#active_model_values)*
                        }
                    }
                }

                struct #marker<T>(std::marker::PhantomData<T>);

                trait #decoder {
                    fn #deserialize(self, json: sea_orm::serde_json::Value)
                        -> Result<ActiveModel, sea_orm::DbErr>;
                }

                impl<T> #decoder for #marker<T>
                where
                    T: sea_orm::serde::de::DeserializeOwned + Into<ActiveModel>,
                {
                    fn #deserialize(self, json: sea_orm::serde_json::Value)
                        -> Result<ActiveModel, sea_orm::DbErr>
                    {
                        sea_orm::serde_json::from_value::<T>(json)
                            .map(Into::into)
                            .map_err(|error| sea_orm::DbErr::Json(error.to_string()))
                    }
                }

                impl<T> #decoder for &#marker<T> {
                    fn #deserialize(self, _json: sea_orm::serde_json::Value)
                        -> Result<ActiveModel, sea_orm::DbErr>
                    {
                        Err(sea_orm::DbErr::Json(
                            "ActiveModel::from_json requires model fields to be independently deserializable"
                                .to_owned(),
                        ))
                    }
                }

                #marker::<#input>(std::marker::PhantomData)
                    .#deserialize(sea_orm::serde_json::Value::Object(object))
            }
        }
    }
}
