use super::active_model::DeriveActiveModel;
use super::attributes::compound_attr;
use super::util::{extract_compound_entity, field_not_ignored_compound, is_compound_field};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::{Attribute, Data, Expr, Fields, LitStr, Type};

pub fn expand_derive_active_model_ex(
    ident: &Ident,
    data: &Data,
    attrs: &[Attribute],
) -> syn::Result<TokenStream> {
    let mut compact = false;
    let mut model_fields = Vec::new();
    let mut ignored_model_fields = Vec::new();
    let mut field_types: Vec<Type> = Vec::new();
    let mut scalar_fields = Vec::new();
    let mut compound_fields = Vec::new();
    let mut belongs_to_fields = Vec::new();
    let mut belongs_to_self_fields = Vec::new();
    let mut has_one_fields = Vec::new();
    let mut has_many_fields = Vec::new();
    let mut has_many_self_fields = Vec::new();
    let mut has_many_via_fields = Vec::new();
    let mut has_many_via_self_fields = Vec::new();

    let (async_, await_) = async_await();

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

    let mut entity_count = HashMap::new();

    if let Data::Struct(item_struct) = &data {
        if let Fields::Named(fields) = &item_struct.fields {
            for field in fields.named.iter() {
                if field.ident.is_some() && field_not_ignored_compound(field) {
                    let field_type = &field.ty;
                    let field_type = quote! { #field_type }
                        .to_string() // e.g.: "Option < String >"
                        .replace(' ', ""); // Remove spaces

                    if is_compound_field(&field_type) {
                        let entity_path = extract_compound_entity(&field_type);
                        *entity_count.entry(entity_path.to_owned()).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    if let Data::Struct(item_struct) = &data {
        if let Fields::Named(fields) = &item_struct.fields {
            for field in fields.named.iter() {
                if let Some(ident) = &field.ident {
                    if field_not_ignored_compound(field) {
                        let field_type = &field.ty;
                        let field_type = quote! { #field_type }
                            .to_string() // e.g.: "Option < String >"
                            .replace(' ', ""); // Remove spaces

                        let ty = if is_compound_field(&field_type) {
                            compound_fields.push(ident);
                            let entity_path = extract_compound_entity(&field_type);

                            if !compact {
                                let compound_attrs =
                                    compound_attr::SeaOrm::from_attributes(&field.attrs)?;
                                if let Some(relation_enum) = compound_attrs.relation_enum {
                                    if compound_attrs.self_ref.is_some()
                                        && compound_attrs.via.is_none()
                                        && compound_attrs.from.is_some()
                                        && compound_attrs.to.is_some()
                                    {
                                        belongs_to_self_fields
                                            .push((ident.clone(), relation_enum.clone()));
                                    } else if compound_attrs.self_ref.is_some()
                                        && compound_attrs.relation_reverse.is_some()
                                        && compound_attrs.via.is_none()
                                        && compound_attrs.from.is_none()
                                        && compound_attrs.to.is_none()
                                    {
                                        has_many_self_fields
                                            .push((ident.clone(), relation_enum.clone()));
                                    }
                                } else if *entity_count.get(entity_path).unwrap() == 1 {
                                    // can only Related to another Entity once
                                    if compound_attrs.belongs_to.is_some() {
                                        belongs_to_fields.push(ident.clone());
                                    } else if compound_attrs.has_one.is_some() {
                                        has_one_fields.push(ident.clone());
                                    } else if compound_attrs.has_many.is_some()
                                        && compound_attrs.via.is_none()
                                    {
                                        has_many_fields.push(ident.clone());
                                    } else if compound_attrs.has_many.is_some()
                                        && compound_attrs.via.is_some()
                                    {
                                        has_many_via_fields.push((
                                            ident.clone(),
                                            compound_attrs.via.as_ref().unwrap().value(),
                                        ));
                                    }
                                }
                                if compound_attrs.self_ref.is_some()
                                    && compound_attrs.via.is_some()
                                    && compound_attrs.reverse.is_none()
                                {
                                    #[allow(clippy::unnecessary_unwrap)]
                                    has_many_via_self_fields.push((
                                        ident.clone(),
                                        compound_attrs.via.as_ref().unwrap().value(),
                                        false,
                                    ));
                                } else if compound_attrs.self_ref.is_some()
                                    && compound_attrs.via.is_some()
                                    && compound_attrs.reverse.is_some()
                                {
                                    #[allow(clippy::unnecessary_unwrap)]
                                    has_many_via_self_fields.push((
                                        ident.clone(),
                                        compound_attrs.via.as_ref().unwrap().value(),
                                        true,
                                    ));
                                }
                            }

                            if field_type.starts_with("HasOne<") {
                                syn::parse_str(&format!("HasOneModel < {entity_path} >"))?
                            } else {
                                syn::parse_str(&format!("HasManyModel < {entity_path} >"))?
                            }
                        } else {
                            scalar_fields.push(ident);
                            syn::parse_str(&format!("sea_orm::ActiveValue < {field_type} >"))?
                        };
                        model_fields.push(ident);
                        field_types.push(ty);
                    } else {
                        ignored_model_fields.push(ident);
                    }
                }
            }
        }
    }

    let active_model_trait_methods =
        DeriveActiveModel::new(ident, data)?.impl_active_model_trait_methods();

    let active_model_action = expand_active_model_action(
        &belongs_to_fields,
        &belongs_to_self_fields,
        &has_one_fields,
        &has_many_fields,
        &has_many_self_fields,
        &has_many_via_fields,
        &has_many_via_self_fields,
    );

    let active_model_setters = expand_active_model_setters(data)?;

    let mut is_changed_expr = quote!(false);

    for field in scalar_fields.iter() {
        is_changed_expr.extend(quote!(|| self.#field.is_set()));
    }
    for field in compound_fields.iter() {
        is_changed_expr.extend(quote!(|| self.#field.is_changed()));
    }

    Ok(quote! {
        #[doc = " Generated by sea-orm-macros"]
        #[derive(Clone, Debug, PartialEq)]
        pub struct ActiveModelEx {
            #(
                #[doc = " Generated by sea-orm-macros"]
                pub #model_fields: #field_types
            ),*
        }

        impl ActiveModel {
            #[doc = " Generated by sea-orm-macros"]
            pub fn into_ex(self) -> ActiveModelEx {
                self.into()
            }
        }

        #[automatically_derived]
        impl sea_orm::ActiveModelTrait for ActiveModelEx {
            type Entity = Entity;

            #active_model_trait_methods

            /// Returns true if any field is set or changed. This is recursive.
            fn is_changed(&self) -> bool {
                #is_changed_expr
            }

            fn default() -> Self {
                <ActiveModel as sea_orm::ActiveModelBehavior>::new().into()
            }
        }

        impl ActiveModelEx {
            #[doc = " Generated by sea-orm-macros"]
            pub fn new() -> Self {
                <Self as sea_orm::ActiveModelTrait>::default()
            }

            #active_model_action

            #active_model_setters
        }

        #[automatically_derived]
        impl std::default::Default for ActiveModelEx {
            fn default() -> Self {
                <Self as sea_orm::ActiveModelTrait>::default()
            }
        }

        #[automatically_derived]
        impl std::convert::From<ActiveModel> for ActiveModelEx {
            fn from(m: ActiveModel) -> Self {
                Self {
                    #(#scalar_fields: m.#scalar_fields,)*
                    #(#compound_fields: Default::default(),)*
                }
            }
        }

        #[automatically_derived]
        impl std::convert::From<ActiveModelEx> for ActiveModel {
            fn from(m: ActiveModelEx) -> Self {
                Self {
                    #(#scalar_fields: m.#scalar_fields,)*
                }
            }
        }

        #[automatically_derived]
        impl std::convert::From<ModelEx> for ActiveModelEx {
            fn from(m: ModelEx) -> Self {
                Self {
                    #(#scalar_fields: sea_orm::ActiveValue::Unchanged(m.#scalar_fields),)*
                    #(#compound_fields: m.#compound_fields.into_active_model(),)*
                }
            }
        }

        #[automatically_derived]
        impl std::convert::TryFrom<ActiveModelEx> for ModelEx {
            type Error = sea_orm::DbErr;
            fn try_from(a: ActiveModelEx) -> Result<Self, sea_orm::DbErr> {
                #(if a.#scalar_fields.is_not_set() {
                    return Err(sea_orm::DbErr::AttrNotSet(stringify!(#scalar_fields).to_owned()));
                })*
                Ok(
                    Self {
                        #(#scalar_fields: a.#scalar_fields.unwrap(),)*
                        #(#compound_fields: a.#compound_fields.try_into_model()?,)*
                        #(#ignored_model_fields: Default::default(),)*
                    }
                )
            }
        }

        #[automatically_derived]
        impl sea_orm::IntoActiveModel<ActiveModelEx> for ModelEx {
            fn into_active_model(self) -> ActiveModelEx {
                self.into()
            }
        }

        #[automatically_derived]
        impl sea_orm::TryIntoModel<ModelEx> for ActiveModelEx {
            fn try_into_model(self) -> Result<ModelEx, sea_orm::DbErr> {
                self.try_into()
            }
        }

        impl Model {
            #[doc = " Generated by sea-orm-macros"]
            pub #async_ fn cascade_delete<'a, C>(self, db: &'a C) -> Result<sea_orm::DeleteResult, sea_orm::DbErr>
            where
                C: sea_orm::TransactionTrait,
            {
                self.into_ex().delete(db)#await_
            }
        }

        impl ModelEx {
            #[doc = " Generated by sea-orm-macros"]
            pub #async_ fn delete<'a, C>(self, db: &'a C) -> Result<sea_orm::DeleteResult, sea_orm::DbErr>
            where
                C: sea_orm::TransactionTrait,
            {
                let active_model: ActiveModelEx = self.into();
                active_model.delete(db)#await_
            }
        }

        impl ActiveModel {
            #[doc = " Generated by sea-orm-macros"]
            pub fn builder() -> ActiveModelEx {
                ActiveModelEx::new()
            }
        }
    })
}

fn expand_active_model_action(
    belongs_to: &[Ident],
    belongs_to_self: &[(Ident, LitStr)],
    has_one: &[Ident],
    has_many: &[Ident],
    has_many_self: &[(Ident, LitStr)],
    has_many_via: &[(Ident, String)],
    has_many_via_self: &[(Ident, String, bool)],
) -> TokenStream {
    let mut belongs_to_action = TokenStream::new();
    let mut belongs_to_after_action = TokenStream::new();
    let mut has_one_before_action = TokenStream::new();
    let mut has_one_action = TokenStream::new();
    let mut has_one_delete = TokenStream::new();
    let mut has_many_before_action = TokenStream::new();
    let mut has_many_action = TokenStream::new();
    let mut has_many_delete = TokenStream::new();
    let mut has_many_via_before_action = TokenStream::new();
    let mut has_many_via_action = TokenStream::new();
    let mut has_many_via_delete = TokenStream::new();

    let (async_, await_) = async_await();
    let box_pin = if cfg!(feature = "async") {
        quote!(Box::pin)
    } else {
        quote!()
    };

    for field in belongs_to {
        belongs_to_action.extend(quote! {
            let #field = if let Some(model) = self.#field.take() {
                if model.is_update() {
                    // has primary key
                    self.set_parent_key(&model)?;
                    if model.is_changed() {
                        let model = #box_pin(model.action(action, db))#await_?;
                        Some(model)
                    } else {
                        Some(model)
                    }
                } else {
                    // new model
                    let model = #box_pin(model.action(action, db))#await_?;
                    self.set_parent_key(&model)?;
                    Some(model)
                }
            } else {
                None
            };
        });

        belongs_to_after_action.extend(quote! {
            if let Some(#field) = #field {
                model.#field = HasOneModel::set(#field);
            }
        });
    }

    for (field, relation_enum) in belongs_to_self {
        let relation_enum = Ident::new(&relation_enum.value(), relation_enum.span());
        let relation_enum = quote!(Relation::#relation_enum);

        // belongs to is the exception where action is performed before self
        belongs_to_action.extend(quote! {
            let #field = if let Some(model) = self.#field.take() {
                if model.is_update() {
                    // has primary key
                    self.set_parent_key_for(&model, #relation_enum)?;
                    if model.is_changed() {
                        let model = #box_pin(model.action(action, db))#await_?;
                        Some(model)
                    } else {
                        Some(model)
                    }
                } else {
                    // new model
                    let model = #box_pin(model.action(action, db))#await_?;
                    self.set_parent_key_for(&model, #relation_enum)?;
                    Some(model)
                }
            } else {
                None
            };
        });

        belongs_to_after_action.extend(quote! {
            if let Some(#field) = #field {
                model.#field = HasOneModel::set(#field);
            }
        });
    }

    let delete_associated_model = quote! {
        let mut item = item.into_active_model();
        if item.clear_parent_key::<Entity>()? {
            item.update(db)#await_?;
        } else {
            deleted.merge(item.into_ex().delete(db)#await_?); // deep delete
        }
    };

    for field in has_one {
        has_one_before_action.extend(quote! {
            let #field = self.#field.take();
        });

        has_one_action.extend(quote! {
            if let Some(mut #field) = #field {
                #field.set_parent_key(&model)?;
                if #field.is_changed() {
                    model.#field = HasOneModel::set(#box_pin(#field.action(action, db))#await_?);
                } else {
                    model.#field = HasOneModel::set(#field);
                }
            }
        });

        has_one_delete.extend(quote! {
            if let Some(item) = self.find_related_of(self.#field.empty_slice()).one(db)#await_? {
                #delete_associated_model
            }
        });
    }

    for field in has_many {
        has_many_before_action.extend(quote! {
            let #field = self.#field.take();
        });

        has_many_action.extend(quote! {
            if #field.is_replace() {
                for item in model.find_related_of(#field.as_slice()).all(db)#await_? {
                    if !#field.find(&item) {
                        #delete_associated_model
                    }
                }
            }
            model.#field = #field.empty_holder();
            for mut #field in #field.into_vec() {
                #field.set_parent_key(&model)?;
                if #field.is_changed() {
                    model.#field.push(#box_pin(#field.action(action, db))#await_?);
                } else {
                    model.#field.push(#field);
                }
            }
        });

        has_many_delete.extend(quote! {
            for item in self.find_related_of(self.#field.as_slice()).all(db)#await_? {
                #delete_associated_model
            }
        });
    }

    for (field, relation_enum) in has_many_self {
        let relation_enum = Ident::new(&relation_enum.value(), relation_enum.span());
        let relation_enum = quote!(Relation::#relation_enum);

        let delete_associated_model = quote! {
            let mut item = item.into_active_model();
            if item.clear_parent_key_for_self_rev(#relation_enum)? {
                item.update(db)#await_?;
            } else {
                // attempt to cascade delete may lead to infinite recursion
                return Err(sea_orm::DbErr::RecordNotUpdated);
            }
        };

        has_many_before_action.extend(quote! {
            let #field = self.#field.take();
        });

        has_many_action.extend(quote! {
            if #field.is_replace() {
                for item in model.find_belongs_to_self(#relation_enum)?.all(db)#await_? {
                    if !#field.find(&item) {
                        #delete_associated_model
                    }
                }
            }
            model.#field = #field.empty_holder();
            for mut #field in #field.into_vec() {
                #field.set_parent_key_for_self_rev(&model, #relation_enum)?;
                if #field.is_changed() {
                    model.#field.push(#box_pin(#field.action(action, db))#await_?);
                } else {
                    model.#field.push(#field);
                }
            }
        });

        has_many_delete.extend(quote! {
            for item in self.find_belongs_to_self(#relation_enum)?.all(db)#await_? {
                #delete_associated_model
            }
        });
    }

    for (field, via_entity) in has_many_via {
        let mut via_entity = via_entity.as_str();
        if let Some((prefix, _)) = via_entity.split_once("::") {
            via_entity = prefix;
        }

        let related_entity: TokenStream = format!("super::{via_entity}::Entity").parse().unwrap();

        has_many_via_before_action.extend(quote! {
            let #field = self.#field.take();
        });

        has_many_via_action.extend(quote! {
            model.#field = #field.empty_holder();
            for item in #field.into_vec() {
                if item.is_update() {
                    // has primary key
                    if item.is_changed() {
                        model.#field.push(#box_pin(item.action(action, db))#await_?);
                    } else {
                        model.#field.push(item);
                    }
                } else {
                    // new model
                    model.#field.push(#box_pin(item.action(action, db))#await_?);
                }
            }
            model.establish_links(
                #related_entity,
                model.#field.as_slice(),
                model.#field.is_replace(),
                db
            )#await_?;
        });

        has_many_via_delete.extend(quote! {
            deleted.merge(self.delete_links(#related_entity, db)#await_?);
        });
    }

    for (field, via_entity, reverse) in has_many_via_self {
        let related_entity: TokenStream = format!("super::{via_entity}::Entity").parse().unwrap();
        let establish_links = Ident::new(
            if *reverse {
                "establish_links_self_rev"
            } else {
                "establish_links_self"
            },
            field.span(),
        );

        has_many_via_before_action.extend(quote! {
            let #field = self.#field.take();
        });

        has_many_via_action.extend(quote! {
            model.#field = #field.empty_holder();
            for item in #field.into_vec() {
                if item.is_update() {
                    // has primary key
                    if item.is_changed() {
                        model.#field.push(#box_pin(item.action(action, db))#await_?);
                    } else {
                        model.#field.push(item);
                    }
                } else {
                    // new model
                    model.#field.push(#box_pin(item.action(action, db))#await_?);
                }
            }
            model.#establish_links(
                #related_entity,
                model.#field.as_slice(),
                model.#field.is_replace(),
                db
            )#await_?;
        });

        has_many_via_delete.extend(quote! {
            deleted.merge(self.delete_links_self(#related_entity, db)#await_?);
        });
    }

    quote! {
        #[doc = " Generated by sea-orm-macros"]
        pub #async_ fn insert<'a, C>(self, db: &'a C) -> Result<ModelEx, sea_orm::DbErr>
        where
            C: sea_orm::TransactionTrait,
        {
            let active_model = self.action(sea_orm::ActiveModelAction::Insert, db)#await_?;
            active_model.try_into()
        }

        #[doc = " Generated by sea-orm-macros"]
        pub #async_ fn update<'a, C>(self, db: &'a C) -> Result<ModelEx, sea_orm::DbErr>
        where
            C: sea_orm::TransactionTrait,
        {
            let active_model = self.action(sea_orm::ActiveModelAction::Update, db)#await_?;
            active_model.try_into()
        }

        #[doc = " Generated by sea-orm-macros"]
        pub #async_ fn save<'a, C>(self, db: &'a C) -> Result<Self, sea_orm::DbErr>
        where
            C: sea_orm::TransactionTrait,
        {
            self.action(sea_orm::ActiveModelAction::Save, db)#await_
        }

        #[doc = " Generated by sea-orm-macros"]
        pub #async_ fn delete<'a, C>(self, db: &'a C) -> Result<sea_orm::DeleteResult, sea_orm::DbErr>
        where
            C: sea_orm::TransactionTrait,
        {
            use sea_orm::{IntoActiveModel, TransactionSession};

            let txn = db.begin()#await_?;
            let db = &txn;
            let mut deleted = sea_orm::DeleteResult::empty();

            #has_one_delete
            #has_many_delete
            #has_many_via_delete

            let model: ActiveModel = self.into();
            deleted.merge(model.delete(db)#await_?);

            txn.commit()#await_?;

            Ok(deleted)
        }

        #[doc = " Generated by sea-orm-macros"]
        pub #async_ fn action<'a, C>(mut self, action: sea_orm::ActiveModelAction, db: &'a C) -> Result<Self, sea_orm::DbErr>
        where
            C: sea_orm::TransactionTrait,
        {
            use sea_orm::{HasOneModel, HasManyModel, IntoActiveModel, TransactionSession};
            let txn = db.begin()#await_?;
            let db = &txn;
            let mut deleted = sea_orm::DeleteResult::empty();

            #belongs_to_action
            #has_one_before_action
            #has_many_before_action
            #has_many_via_before_action

            let model: ActiveModel = self.into();

            let mut model: Self = if model.is_changed() {
                match action {
                    sea_orm::ActiveModelAction::Insert => model.insert(db)#await_,
                    sea_orm::ActiveModelAction::Update => model.update(db)#await_,
                    sea_orm::ActiveModelAction::Save => if !model.is_update() {
                        model.insert(db)#await_
                    } else {
                        model.update(db)#await_
                    },
                }?.into_ex().into()
            } else {
                model.into()
            };

            #belongs_to_after_action
            #has_one_action
            #has_many_action
            #has_many_via_action

            txn.commit()#await_?;

            Ok(model)
        }
    }
}

fn expand_active_model_setters(data: &Data) -> syn::Result<TokenStream> {
    let mut setters = TokenStream::new();

    if let Data::Struct(item_struct) = &data {
        if let Fields::Named(fields) = &item_struct.fields {
            for field in &fields.named {
                if let Some(ident) = &field.ident {
                    let field_type = &field.ty;
                    let field_type_str = quote! { #field_type }
                        .to_string() // e.g.: "Option < String >"
                        .replace(' ', ""); // Remove spaces

                    let mut ignore = false;

                    for attr in field.attrs.iter() {
                        if attr.path().is_ident("sea_orm") {
                            attr.parse_nested_meta(|meta| {
                                if meta.path.is_ident("ignore") {
                                    ignore = true;
                                } else {
                                    // Reads the value expression to advance the parse stream.
                                    let _: Option<Expr> = meta.value().and_then(|v| v.parse()).ok();
                                }

                                Ok(())
                            })?;
                        }
                    }

                    if ignore {
                        continue;
                    }

                    if is_compound_field(&field_type_str) {
                        let entity_path = extract_compound_entity(&field_type_str);
                        let active_model_type: Type = syn::parse_str(&format!(
                            "{}ActiveModelEx",
                            entity_path.trim_end_matches("Entity")
                        ))?;

                        if field_type_str.starts_with("HasOne<") {
                            let setter = format_ident!("set_{}", ident);

                            setters.extend(quote! {
                                #[doc = " Generated by sea-orm-macros"]
                                pub fn #setter(mut self, v: impl Into<#active_model_type>) -> Self {
                                    self.#ident.replace(v.into());
                                    self
                                }
                            });
                        } else {
                            let setter = format_ident!(
                                "add_{}",
                                pluralizer::pluralize(&ident.to_string(), 1, false)
                            );

                            setters.extend(quote! {
                                #[doc = " Generated by sea-orm-macros"]
                                pub fn #setter(mut self, v: impl Into<#active_model_type>) -> Self {
                                    self.#ident.push(v.into());
                                    self
                                }
                            });
                        }
                    } else {
                        let setter = format_ident!("set_{}", ident);

                        setters.extend(quote! {
                            #[doc = " Generated by sea-orm-macros"]
                            pub fn #setter(mut self, v: impl Into<#field_type>) -> Self {
                                self.#ident = sea_orm::Set(v.into());
                                self
                            }
                        });
                    }
                }
            }
        }
    }

    Ok(setters)
}

fn async_await() -> (TokenStream, TokenStream) {
    if cfg!(feature = "async") {
        (quote!(async), quote!(.await))
    } else {
        (quote!(), quote!())
    }
}
