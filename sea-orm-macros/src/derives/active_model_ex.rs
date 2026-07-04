use super::active_model::DeriveActiveModel;
use super::attributes::compound_attr;
use super::util::{CardinalityKind, CompoundKind, CompoundType, consume_meta};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::{Attribute, Data, Field, Fields, LitStr, PathArguments, Type, TypePath, Visibility};

fn async_await() -> (TokenStream, TokenStream) {
    if cfg!(feature = "async") {
        (quote!(async), quote!(.await))
    } else {
        (quote!(), quote!())
    }
}

enum ActiveModelExRelationAttr {
    BelongsTo {
        cardinality: CardinalityKind,
    },
    BelongsToSelf {
        relation_enum: LitStr,
        cardinality: CardinalityKind,
    },
    HasOne {
        cardinality: CardinalityKind,
    },
    HasMany,
    HasManySelf {
        relation_enum: LitStr,
    },
    HasManyVia {
        via: LitStr,
    },
}

struct ActiveModelExSelfViaRelationAttr {
    via: LitStr,
    reverse: bool,
}

#[derive(Default)]
struct ActiveModelExRelationAttrs {
    relation: Option<ActiveModelExRelationAttr>,
    self_via: Option<ActiveModelExSelfViaRelationAttr>,
}

impl ActiveModelExRelationAttrs {
    fn from_attrs(
        field_attrs: &[Attribute],
        ident: &Ident,
        compound: &CompoundType,
    ) -> syn::Result<Self> {
        let attrs = compound_attr::SeaOrm::try_from_attributes(field_attrs)?.unwrap_or_default();

        let relation = match &attrs {
            compound_attr::SeaOrm {
                relation_enum: Some(relation_enum),
                self_ref: Some(_),
                via: None,
                from: Some(_),
                to: Some(_),
                ..
            } => Some(ActiveModelExRelationAttr::BelongsToSelf {
                relation_enum: relation_enum.clone(),
                cardinality: compound.cardinality().ok_or_else(|| {
                    syn::Error::new_spanned(ident, "self_ref belongs_to must be paired with HasOne")
                })?,
            }),
            compound_attr::SeaOrm {
                relation_enum: Some(relation_enum),
                self_ref: Some(_),
                relation_reverse: Some(_),
                via: None,
                from: None,
                to: None,
                ..
            } => Some(ActiveModelExRelationAttr::HasManySelf {
                relation_enum: relation_enum.clone(),
            }),
            compound_attr::SeaOrm {
                relation_enum: Some(_),
                ..
            } => None,
            compound_attr::SeaOrm {
                belongs_to: Some(_),
                ..
            } => Some(ActiveModelExRelationAttr::BelongsTo {
                cardinality: compound.cardinality().ok_or_else(|| {
                    syn::Error::new_spanned(ident, "belongs_to must be paired with HasOne")
                })?,
            }),
            compound_attr::SeaOrm {
                has_one: Some(_), ..
            } => {
                let cardinality = match compound.kind {
                    CompoundKind::HasOne(CardinalityKind::Optional) => CardinalityKind::Optional,
                    CompoundKind::HasOne(CardinalityKind::Required) => {
                        return Err(syn::Error::new_spanned(
                            ident,
                            "#[sea_orm(has_one)] must be paired with HasOne<Option<Entity>>",
                        ));
                    }
                    CompoundKind::HasMany => {
                        return Err(syn::Error::new_spanned(
                            ident,
                            "#[sea_orm(has_one)] must be paired with HasOne<Option<Entity>>",
                        ));
                    }
                };
                Some(ActiveModelExRelationAttr::HasOne { cardinality })
            }
            compound_attr::SeaOrm {
                has_many: Some(_),
                via: None,
                ..
            } => Some(ActiveModelExRelationAttr::HasMany),
            compound_attr::SeaOrm {
                has_many: Some(_),
                via: Some(via),
                ..
            } => Some(ActiveModelExRelationAttr::HasManyVia { via: via.clone() }),
            _ => None,
        };

        let self_via = if attrs.self_ref.is_some() {
            attrs
                .via
                .as_ref()
                .map(|via| ActiveModelExSelfViaRelationAttr {
                    via: via.clone(),
                    reverse: attrs.reverse.is_some(),
                })
        } else {
            None
        };

        Ok(Self { relation, self_via })
    }
}

enum ActiveModelExFieldKind<'a> {
    Ignored,
    Scalar { ty: &'a Type },
    Compound { compound: CompoundType },
}

struct ActiveModelExFieldAttrs {
    ignore: bool,
}

impl ActiveModelExFieldAttrs {
    fn from_field(field: &Field) -> syn::Result<Self> {
        let mut this = Self { ignore: false };

        for attr in &field.attrs {
            if !attr.path().is_ident("sea_orm") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("ignore") {
                    this.ignore = true;
                } else {
                    consume_meta(meta);
                }
                Ok(())
            })?;
        }

        Ok(this)
    }
}

struct ActiveModelExField<'a> {
    ident: &'a Ident,
    attrs: &'a [Attribute],
    kind: ActiveModelExFieldKind<'a>,
}

impl<'a> ActiveModelExField<'a> {
    fn from_field(field: &'a Field) -> syn::Result<Self> {
        let Some(ident) = &field.ident else {
            return Err(syn::Error::new_spanned(field, "expected named field"));
        };
        let attrs = ActiveModelExFieldAttrs::from_field(field)?;
        let kind = if attrs.ignore {
            ActiveModelExFieldKind::Ignored
        } else if let Type::Path(type_path) = &field.ty
            && let Some(compound) = CompoundType::from_type(type_path)?
        {
            ActiveModelExFieldKind::Compound { compound }
        } else {
            ActiveModelExFieldKind::Scalar { ty: &field.ty }
        };
        Ok(Self {
            ident,
            attrs: &field.attrs,
            kind,
        })
    }

    fn setter(&self) -> syn::Result<TokenStream> {
        let ident = self.ident;
        match &self.kind {
            ActiveModelExFieldKind::Ignored => Ok(quote!()),
            ActiveModelExFieldKind::Scalar { ty } => {
                let field_type = ty;
                let setter = format_ident!("set_{}", ident);

                Ok(quote! {
                    #[doc = " Generated by sea-orm-macros"]
                    pub fn #setter(mut self, v: impl Into<#field_type>) -> Self {
                        self.#ident = sea_orm::Set(v.into());
                        self
                    }
                })
            }
            ActiveModelExFieldKind::Compound { compound, .. } => {
                let entity_path = &compound.entity;
                let mut active_model_path = entity_path.path.clone();
                let Some(segment) = active_model_path.segments.last_mut() else {
                    return Err(syn::Error::new_spanned(entity_path, "expected entity path"));
                };
                segment.ident = format_ident!("ActiveModelEx");
                segment.arguments = PathArguments::None;

                match compound.kind {
                    CompoundKind::HasOne(cardinality) => {
                        let has_one_target = cardinality.has_one_target_type(quote!(#entity_path));
                        let setter = format_ident!("set_{}", ident);
                        let optional = if cardinality.is_optional() {
                            quote!(Some)
                        } else {
                            quote!()
                        };
                        let optional_setters = if cardinality.is_optional() {
                            let optional_setter = format_ident!("set_{}_option", ident);
                            let clear_method = format_ident!("clear_{}", ident);

                            quote! {
                                #[doc = " Generated by sea-orm-macros"]
                                pub fn #optional_setter(mut self, v: Option<impl Into<#active_model_path>>) -> Self {
                                    self.#ident = sea_orm::ActiveHasOne::<#has_one_target>::set(v);
                                    self
                                }

                                #[doc = " Generated by sea-orm-macros"]
                                pub fn #clear_method(mut self) -> Self {
                                    self.#ident = sea_orm::ActiveHasOne::Set(None);
                                    self
                                }
                            }
                        } else {
                            quote!()
                        };

                        Ok(quote! {
                            #[doc = " Generated by sea-orm-macros"]
                            pub fn #setter(mut self, v: impl Into<#active_model_path>) -> Self {
                                self.#ident = sea_orm::ActiveHasOne::<#has_one_target>::set(#optional(v.into()));
                                self
                            }

                            #optional_setters
                        })
                    }
                    CompoundKind::HasMany => {
                        let setter = format_ident!(
                            "add_{}",
                            pluralizer::pluralize(&ident.to_string(), 1, false)
                        );

                        Ok(quote! {
                            #[doc = " Generated by sea-orm-macros"]
                            pub fn #setter(mut self, v: impl Into<#active_model_path>) -> Self {
                                self.#ident.push(v.into());
                                self
                            }
                        })
                    }
                }
            }
        }
    }

    fn expand_into(&self, output: &mut ActiveModelExOutput<'a>) -> syn::Result<()> {
        let ident = self.ident;
        match &self.kind {
            ActiveModelExFieldKind::Ignored => {
                output.ignored_model_fields.push(ident);
            }
            ActiveModelExFieldKind::Scalar { ty } => {
                let field_type = ty;
                output.model_field_defs.push(quote! {
                    #[doc = " Generated by sea-orm-macros"]
                    pub #ident: sea_orm::ActiveValue<#field_type>
                });
                output.active_model_setters.extend(self.setter()?);
                output.is_changed_exprs.push(quote!(self.#ident.is_set()));
                output.scalar_fields.push(ident);
            }
            ActiveModelExFieldKind::Compound { compound } => {
                let entity_type = &compound.entity;
                let field_type = match compound.kind {
                    CompoundKind::HasOne(cardinality) => {
                        let target_type = cardinality.has_one_target_type(quote!(#entity_type));
                        quote!(ActiveHasOne<#target_type>)
                    }
                    CompoundKind::HasMany => quote!(ActiveHasMany<#entity_type>),
                };
                output.model_field_defs.push(quote! {
                    #[doc = " Generated by sea-orm-macros"]
                    pub #ident: #field_type
                });
                output.active_model_setters.extend(self.setter()?);
                output
                    .is_changed_exprs
                    .push(quote!(self.#ident.is_changed()));
                output.compound_fields.push(ident);
            }
        }

        Ok(())
    }
}

enum ActiveModelExRelation<'a> {
    BelongsTo(ActiveHasOneField<'a>),
    BelongsToSelf(SelfHasOneField<'a>),
    HasOne(ActiveHasOneField<'a>),
    HasMany(HasManyField<'a>),
    HasManySelf(SelfRelationField<'a>),
    HasManyVia(ViaField<'a>),
    HasManyViaSelf(SelfViaField<'a>),
}

#[derive(Default)]
struct ActiveModelActionTokens {
    belongs_to_action: TokenStream,
    belongs_to_after_action: TokenStream,
    has_one_before_action: TokenStream,
    has_one_action: TokenStream,
    has_one_delete: TokenStream,
    has_many_before_action: TokenStream,
    has_many_action: TokenStream,
    has_many_delete: TokenStream,
    has_many_via_before_action: TokenStream,
    has_many_via_action: TokenStream,
    has_many_via_delete: TokenStream,
}

impl ActiveModelActionTokens {
    fn from_dense_fields(fields: &[ActiveModelExField<'_>]) -> syn::Result<Self> {
        let entity_count = fields
            .iter()
            .filter_map(|field| {
                if let ActiveModelExFieldKind::Compound { compound, .. } = &field.kind {
                    Some(&compound.entity)
                } else {
                    None
                }
            })
            .fold(
                HashMap::<&TypePath, usize>::new(),
                |mut entity_count, entity| {
                    *entity_count.entry(entity).or_insert(0) += 1;
                    entity_count
                },
            );

        let relations = fields
            .iter()
            .filter_map(|field| {
                let ActiveModelExFieldKind::Compound { compound } = &field.kind else {
                    return None;
                };
                Some((field.ident, field.attrs, compound))
            })
            .try_fold(Vec::new(), |mut relations, (ident, attrs, compound)| {
                let relation_attrs =
                    ActiveModelExRelationAttrs::from_attrs(attrs, ident, compound)?;
                let is_unique_entity = entity_count
                    .get(&compound.entity)
                    .is_some_and(|count| *count == 1);

                let relation = match &relation_attrs.relation {
                    Some(ActiveModelExRelationAttr::BelongsToSelf {
                        relation_enum,
                        cardinality,
                    }) => Some(ActiveModelExRelation::BelongsToSelf(SelfHasOneField {
                        ident,
                        relation_enum: relation_enum.clone(),
                        cardinality: *cardinality,
                    })),
                    Some(ActiveModelExRelationAttr::HasManySelf { relation_enum }) => {
                        Some(ActiveModelExRelation::HasManySelf(SelfRelationField {
                            ident,
                            relation_enum: relation_enum.clone(),
                        }))
                    }
                    Some(ActiveModelExRelationAttr::BelongsTo { cardinality })
                        if is_unique_entity =>
                    {
                        Some(ActiveModelExRelation::BelongsTo(ActiveHasOneField {
                            ident,
                            entity: &compound.entity,
                            cardinality: *cardinality,
                        }))
                    }
                    Some(ActiveModelExRelationAttr::HasOne { cardinality }) if is_unique_entity => {
                        Some(ActiveModelExRelation::HasOne(ActiveHasOneField {
                            ident,
                            entity: &compound.entity,
                            cardinality: *cardinality,
                        }))
                    }
                    Some(ActiveModelExRelationAttr::HasMany) if is_unique_entity => {
                        Some(ActiveModelExRelation::HasMany(HasManyField {
                            ident,
                            entity: &compound.entity,
                        }))
                    }
                    Some(ActiveModelExRelationAttr::HasManyVia { via }) if is_unique_entity => {
                        Some(ActiveModelExRelation::HasManyVia(ViaField {
                            ident,
                            entity: via.value(),
                        }))
                    }
                    _ => None,
                };
                let self_via_relation = relation_attrs.self_via.as_ref().map(|self_via| {
                    ActiveModelExRelation::HasManyViaSelf(SelfViaField {
                        ident,
                        entity: self_via.via.value(),
                        reverse: self_via.reverse,
                    })
                });

                relations.extend(relation.into_iter().chain(self_via_relation));
                Ok::<_, syn::Error>(relations)
            })?;

        let mut this = Self::default();
        for relation in &relations {
            relation.expand_into(&mut this);
        }

        Ok(this)
    }

    fn expand(self) -> TokenStream {
        let (async_, await_) = async_await();
        let Self {
            belongs_to_action,
            belongs_to_after_action,
            has_one_before_action,
            has_one_action,
            has_one_delete,
            has_many_before_action,
            has_many_action,
            has_many_delete,
            has_many_via_before_action,
            has_many_via_action,
            has_many_via_delete,
        } = self;

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
                use sea_orm::{ActiveHasOne, ActiveHasMany, IntoActiveModel, TransactionSession};
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
}

#[derive(Default)]
struct ActiveModelExOutput<'a> {
    model_field_defs: Vec<TokenStream>,
    active_model_setters: TokenStream,
    is_changed_exprs: Vec<TokenStream>,
    ignored_model_fields: Vec<&'a Ident>,
    scalar_fields: Vec<&'a Ident>,
    compound_fields: Vec<&'a Ident>,
}

impl<'a> ActiveModelExOutput<'a> {
    fn from_fields(fields: &'a [ActiveModelExField<'a>]) -> syn::Result<Self> {
        let mut this = Self::default();

        for field in fields {
            field.expand_into(&mut this)?;
        }

        Ok(this)
    }
}

fn expand_active_model_ex<'a>(
    vis: &Visibility,
    ident: &Ident,
    data: &Data,
    fields: &'a [ActiveModelExField<'a>],
    active_model_action: TokenStream,
) -> syn::Result<TokenStream> {
    let (async_, await_) = async_await();
    let active_model_trait_methods =
        DeriveActiveModel::new(vis, ident, data)?.impl_active_model_trait_methods();
    let ActiveModelExOutput {
        model_field_defs,
        active_model_setters,
        is_changed_exprs,
        ignored_model_fields,
        scalar_fields,
        compound_fields,
    } = ActiveModelExOutput::from_fields(fields)?;

    Ok(quote! {
        #[doc = " Generated by sea-orm-macros"]
        #[derive(Clone, Debug, PartialEq)]
        #vis struct ActiveModelEx {
            #(#model_field_defs),*
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
                false #(|| #is_changed_exprs)*
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

struct ActiveHasOneField<'a> {
    ident: &'a Ident,
    entity: &'a TypePath,
    cardinality: CardinalityKind,
}

impl ActiveHasOneField<'_> {
    fn belongs_to_action(&self) -> TokenStream {
        let (_, await_) = async_await();
        let box_pin = if cfg!(feature = "async") {
            quote!(Box::pin)
        } else {
            quote!()
        };
        let ident = self.ident;
        let related_entity = self.entity;
        let has_one_target = self
            .cardinality
            .has_one_target_type(quote!(#related_entity));
        let optional = if self.cardinality.is_optional() {
            quote!(Some)
        } else {
            quote!()
        };
        let replace_model = quote! {
            ActiveHasOne::Set(#optional(model)) => {
                let mut model = *model;
                if model.is_update() {
                    self.set_parent_key(&model)?;
                    if model.is_changed() {
                        model = #box_pin(model.action(action, db))#await_?;
                    }
                } else {
                    model = #box_pin(model.action(action, db))#await_?;
                    self.set_parent_key(&model)?;
                }
                ActiveHasOne::<#has_one_target>::set(#optional(model))
            }
        };
        let clear_optional = if self.cardinality.is_optional() {
            quote! {
                ActiveHasOne::Set(None) => {
                    if !self.clear_parent_key::<#related_entity>()? {
                        return Err(sea_orm::DbErr::Type(format!(
                            "Relation {} cannot be cleared",
                            stringify!(#ident)
                        )));
                    }
                    ActiveHasOne::Set(None)
                }
            }
        } else {
            quote!()
        };

        quote! {
            let #ident = match self.#ident.take() {
                ActiveHasOne::NotSet => ActiveHasOne::NotSet,
                #replace_model
                #clear_optional
            };
        }
    }

    fn belongs_to_after_action(&self) -> TokenStream {
        let ident = self.ident;
        quote! {
            if #ident.is_set() {
                model.#ident = #ident;
            }
        }
    }

    fn has_one_before_action(&self) -> TokenStream {
        let ident = self.ident;
        quote! {
            let #ident = std::mem::take(&mut self.#ident);
        }
    }

    fn has_one_action(&self) -> TokenStream {
        let (_, await_) = async_await();
        let box_pin = if cfg!(feature = "async") {
            quote!(Box::pin)
        } else {
            quote!()
        };
        let delete_associated_model = quote! {
            let mut item = item.into_active_model();
            if item.clear_parent_key::<Entity>()? {
                item.update(db)#await_?;
            } else {
                deleted.merge(item.into_ex().delete(db)#await_?); // deep delete
            }
        };
        let ident = self.ident;
        let related_entity = self.entity;
        let has_one_target = self
            .cardinality
            .has_one_target_type(quote!(#related_entity));

        let optional = if self.cardinality.is_optional() {
            quote!(Some)
        } else {
            quote!()
        };

        let delete_existing_child = quote! {
            if let Some(item) = model.find_related(#related_entity).one(db)#await_? {
                let item_pk = sea_orm::ModelTrait::get_primary_key_value(&item);
                let child_pk = sea_orm::ActiveModelTrait::get_primary_key_value(&child);
                if child_pk.as_ref() != Some(&item_pk) {
                    #delete_associated_model
                }
            }
        };
        let set_child_action = quote! {
            #delete_existing_child
            child.set_parent_key(&model)?;
            let child = if child.is_changed() {
                #box_pin(child.action(action, db))#await_?
            } else {
                child
            };
            model.#ident = ActiveHasOne::<#has_one_target>::set(#optional(child));
        };
        let set_relation_action = if self.cardinality.is_optional() {
            quote! {
                ActiveHasOne::Set(Some(child)) => {
                    let mut child = *child;
                    #set_child_action
                }
                ActiveHasOne::Set(None) => {
                    if let Some(item) = model.find_related(#related_entity).one(db)#await_? {
                        #delete_associated_model
                    }
                    model.#ident = ActiveHasOne::Set(None);
                }
            }
        } else {
            quote! {
                ActiveHasOne::Set(child) => {
                    let mut child = *child;
                    #set_child_action
                }
            }
        };

        quote! {
            match #ident {
                ActiveHasOne::NotSet => {}
                #set_relation_action
            }
        }
    }

    fn has_one_delete(&self) -> TokenStream {
        let (_, await_) = async_await();
        let related_entity = self.entity;
        let delete_associated_model = quote! {
            let mut item = item.into_active_model();
            if item.clear_parent_key::<Entity>()? {
                item.update(db)#await_?;
            } else {
                deleted.merge(item.into_ex().delete(db)#await_?); // deep delete
            }
        };

        quote! {
            if let Some(item) = self.find_related(#related_entity).one(db)#await_? {
                #delete_associated_model
            }
        }
    }

    fn expand_belongs_to_into(&self, output: &mut ActiveModelActionTokens) {
        output.belongs_to_action.extend(self.belongs_to_action());
        output
            .belongs_to_after_action
            .extend(self.belongs_to_after_action());
    }

    fn expand_has_one_into(&self, output: &mut ActiveModelActionTokens) {
        output
            .has_one_before_action
            .extend(self.has_one_before_action());
        output.has_one_action.extend(self.has_one_action());
        output.has_one_delete.extend(self.has_one_delete());
    }
}

struct SelfHasOneField<'a> {
    ident: &'a Ident,
    relation_enum: LitStr,
    cardinality: CardinalityKind,
}

impl SelfHasOneField<'_> {
    fn belongs_to_action(&self) -> TokenStream {
        let (_, await_) = async_await();
        let box_pin = if cfg!(feature = "async") {
            quote!(Box::pin)
        } else {
            quote!()
        };
        let ident = self.ident;
        let relation_enum = Ident::new(&self.relation_enum.value(), self.relation_enum.span());
        let relation_enum = quote!(Relation::#relation_enum);
        let has_one_target = self.cardinality.has_one_target_type(quote!(Entity));
        let optional = if self.cardinality.is_optional() {
            quote!(Some)
        } else {
            quote!()
        };
        let replace_model = quote! {
            ActiveHasOne::Set(#optional(model)) => {
                let mut model = *model;
                if model.is_update() {
                    self.set_parent_key_for(&model, #relation_enum)?;
                    if model.is_changed() {
                        model = #box_pin(model.action(action, db))#await_?;
                    }
                } else {
                    model = #box_pin(model.action(action, db))#await_?;
                    self.set_parent_key_for(&model, #relation_enum)?;
                }
                ActiveHasOne::<#has_one_target>::set(#optional(model))
            }
        };
        let clear_optional = if self.cardinality.is_optional() {
            quote! {
                ActiveHasOne::Set(None) => {
                    if !self.clear_parent_key_for(#relation_enum)? {
                        return Err(sea_orm::DbErr::Type(format!(
                            "Relation {} cannot be cleared",
                            stringify!(#ident)
                        )));
                    }
                    ActiveHasOne::Set(None)
                }
            }
        } else {
            quote!()
        };

        quote! {
            let #ident = match self.#ident.take() {
                ActiveHasOne::NotSet => ActiveHasOne::NotSet,
                #replace_model
                #clear_optional
            };
        }
    }

    fn belongs_to_after_action(&self) -> TokenStream {
        let ident = self.ident;
        quote! {
            if #ident.is_set() {
                model.#ident = #ident;
            }
        }
    }

    fn expand_belongs_to_self_into(&self, output: &mut ActiveModelActionTokens) {
        output.belongs_to_action.extend(self.belongs_to_action());
        output
            .belongs_to_after_action
            .extend(self.belongs_to_after_action());
    }
}

struct SelfRelationField<'a> {
    ident: &'a Ident,
    relation_enum: LitStr,
}

impl SelfRelationField<'_> {
    fn expand_into(&self, output: &mut ActiveModelActionTokens) {
        let (_, await_) = async_await();
        let box_pin = if cfg!(feature = "async") {
            quote!(Box::pin)
        } else {
            quote!()
        };
        let ident = self.ident;
        let relation_enum = Ident::new(&self.relation_enum.value(), self.relation_enum.span());
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

        let has_many_before_action = quote! {
            let #ident = self.#ident.take();
        };

        let has_many_action = quote! {
            if #ident.is_replace() {
                for item in model.find_belongs_to_self(#relation_enum, db.get_database_backend())?.all(db)#await_? {
                    if !#ident.find(&item) {
                        #delete_associated_model
                    }
                }
            }
            model.#ident = #ident.empty_holder();
            for mut #ident in #ident.into_vec() {
                #ident.set_parent_key_for_self_rev(&model, #relation_enum)?;
                let #ident = if #ident.is_changed() {
                    #box_pin(#ident.action(action, db))#await_?
                } else {
                    #ident
                };
                model.#ident.push(#ident);
            }
        };

        let has_many_delete = quote! {
            for item in self.find_belongs_to_self(#relation_enum, db.get_database_backend())?.all(db)#await_? {
                #delete_associated_model
            }
        };

        output.has_many_before_action.extend(has_many_before_action);
        output.has_many_action.extend(has_many_action);
        output.has_many_delete.extend(has_many_delete);
    }
}

struct ViaField<'a> {
    ident: &'a Ident,
    entity: String,
}

impl ViaField<'_> {
    fn expand_into(&self, output: &mut ActiveModelActionTokens) {
        let (_, await_) = async_await();
        let box_pin = if cfg!(feature = "async") {
            quote!(Box::pin)
        } else {
            quote!()
        };
        let ident = self.ident;
        let mut via_entity = self.entity.as_str();
        if let Some((prefix, _)) = via_entity.split_once("::") {
            via_entity = prefix;
        }

        let related_entity: TokenStream = format!("super::{via_entity}::Entity").parse().unwrap();

        let has_many_via_before_action = quote! {
            let #ident = self.#ident.take();
        };

        let has_many_via_action = quote! {
            model.#ident = #ident.empty_holder();
            for item in #ident.into_vec() {
                let item = if item.is_update() && !item.is_changed() {
                    item
                } else {
                    #box_pin(item.action(action, db))#await_?
                };
                model.#ident.push(item);
            }
            model.establish_links(
                #related_entity,
                model.#ident.as_slice(),
                model.#ident.is_replace(),
                db
            )#await_?;
        };

        let has_many_via_delete = quote! {
            deleted.merge(self.delete_links(#related_entity, db)#await_?);
        };

        output
            .has_many_via_before_action
            .extend(has_many_via_before_action);
        output.has_many_via_action.extend(has_many_via_action);
        output.has_many_via_delete.extend(has_many_via_delete);
    }
}

struct SelfViaField<'a> {
    ident: &'a Ident,
    entity: String,
    reverse: bool,
}

impl SelfViaField<'_> {
    fn expand_into(&self, output: &mut ActiveModelActionTokens) {
        let (_, await_) = async_await();
        let box_pin = if cfg!(feature = "async") {
            quote!(Box::pin)
        } else {
            quote!()
        };
        let ident = self.ident;
        let related_entity: TokenStream =
            format!("super::{}::Entity", self.entity).parse().unwrap();
        let establish_links = Ident::new(
            if self.reverse {
                "establish_links_self_rev"
            } else {
                "establish_links_self"
            },
            ident.span(),
        );

        let has_many_via_before_action = quote! {
            let #ident = self.#ident.take();
        };

        let has_many_via_action = quote! {
            model.#ident = #ident.empty_holder();
            for item in #ident.into_vec() {
                let item = if item.is_update() && !item.is_changed() {
                    item
                } else {
                    #box_pin(item.action(action, db))#await_?
                };
                model.#ident.push(item);
            }
            model.#establish_links(
                #related_entity,
                model.#ident.as_slice(),
                model.#ident.is_replace(),
                db
            )#await_?;
        };

        let has_many_via_delete = quote! {
            deleted.merge(self.delete_links_self(#related_entity, db)#await_?);
        };

        output
            .has_many_via_before_action
            .extend(has_many_via_before_action);
        output.has_many_via_action.extend(has_many_via_action);
        output.has_many_via_delete.extend(has_many_via_delete);
    }
}

struct HasManyField<'a> {
    ident: &'a Ident,
    entity: &'a TypePath,
}

impl HasManyField<'_> {
    fn has_many_before_action(&self) -> TokenStream {
        let ident = self.ident;
        quote! {
            let #ident = self.#ident.take();
        }
    }

    fn has_many_action(&self) -> TokenStream {
        let (_, await_) = async_await();
        let box_pin = if cfg!(feature = "async") {
            quote!(Box::pin)
        } else {
            quote!()
        };
        let ident = self.ident;
        let related_entity = self.entity;
        let delete_associated_model = quote! {
            let mut item = item.into_active_model();
            if item.clear_parent_key::<Entity>()? {
                item.update(db)#await_?;
            } else {
                deleted.merge(item.into_ex().delete(db)#await_?); // deep delete
            }
        };
        quote! {
            if #ident.is_replace() {
                for item in model.find_related(#related_entity).all(db)#await_? {
                    if !#ident.find(&item) {
                        #delete_associated_model
                    }
                }
            }
            model.#ident = #ident.empty_holder();
            for mut #ident in #ident.into_vec() {
                #ident.set_parent_key(&model)?;
                let #ident = if #ident.is_changed() {
                    #box_pin(#ident.action(action, db))#await_?
                } else {
                    #ident
                };
                model.#ident.push(#ident);
            }
        }
    }

    fn has_many_delete(&self) -> TokenStream {
        let (_, await_) = async_await();
        let related_entity = self.entity;
        let delete_associated_model = quote! {
            let mut item = item.into_active_model();
            if item.clear_parent_key::<Entity>()? {
                item.update(db)#await_?;
            } else {
                deleted.merge(item.into_ex().delete(db)#await_?); // deep delete
            }
        };
        quote! {
            for item in self.find_related(#related_entity).all(db)#await_? {
                #delete_associated_model
            }
        }
    }

    fn expand_has_many_into(&self, output: &mut ActiveModelActionTokens) {
        output
            .has_many_before_action
            .extend(self.has_many_before_action());
        output.has_many_action.extend(self.has_many_action());
        output.has_many_delete.extend(self.has_many_delete());
    }
}

impl ActiveModelExRelation<'_> {
    fn expand_into(&self, output: &mut ActiveModelActionTokens) {
        match self {
            Self::BelongsTo(field) => field.expand_belongs_to_into(output),
            Self::BelongsToSelf(field) => field.expand_belongs_to_self_into(output),
            Self::HasOne(field) => field.expand_has_one_into(output),
            Self::HasMany(field) => field.expand_has_many_into(output),
            Self::HasManySelf(field) => field.expand_into(output),
            Self::HasManyVia(field) => field.expand_into(output),
            Self::HasManyViaSelf(field) => field.expand_into(output),
        }
    }
}

pub fn expand_derive_active_model_ex(
    vis: &Visibility,
    ident: &Ident,
    data: &Data,
    attrs: &[Attribute],
) -> syn::Result<TokenStream> {
    let mut compact = false;

    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("sea_orm"))
        .try_for_each(|attr| {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("compact_model") {
                    compact = true;
                } else {
                    consume_meta(meta);
                }
                Ok(())
            })
        })?;

    let fields = if let Data::Struct(r#struct) = data
        && let Fields::Named(fields) = &r#struct.fields
    {
        fields
            .named
            .iter()
            .map(ActiveModelExField::from_field)
            .collect::<syn::Result<Vec<_>>>()?
    } else {
        return Err(syn::Error::new_spanned(
            ident,
            "You can only derive DeriveActiveModelEx on structs",
        ));
    };
    let active_model_action_tokens = if compact {
        ActiveModelActionTokens::default()
    } else {
        ActiveModelActionTokens::from_dense_fields(&fields)?
    };
    let active_model_action = active_model_action_tokens.expand();

    expand_active_model_ex(vis, ident, data, &fields, active_model_action)
}
