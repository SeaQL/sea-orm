use super::active_model::DeriveActiveModel;
use super::attributes::compound_attr;
use super::model_ex::infer_relation_name_from_entity;
use super::util::{
    CardinalityKind, CompoundKind, CompoundType, Junction, RelationColumns, async_token,
    await_token, consume_meta, escape_rust_keyword, is_self_entity, trim_starting_raw_identifier,
};
use heck::ToUpperCamelCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{Attribute, Data, LitStr, PathArguments, Type, TypePath, Visibility};

struct BelongsToAttr {
    cardinality: CardinalityKind,
    from: RelationColumns,
}

enum RelationAttr {
    BelongsTo {
        attr: BelongsToAttr,
        /// Explicit relation disambiguator, present when more than one relation
        /// targets the same entity. Routes the FK write through the
        /// relation-keyed `*_parent_key_for` helpers instead of the entity-keyed ones.
        relation_enum: Option<LitStr>,
    },
    BelongsToSelf {
        attr: BelongsToAttr,
        relation_enum: LitStr,
    },
    HasOne,
    HasMany,
    HasManySelf {
        relation_enum: LitStr,
    },
    ManyToMany {
        junction: Junction,
    },
    ManyToManySelf {
        junction_module: Ident,
        reverse: bool,
    },
}

impl RelationAttr {
    fn from_attr(
        attrs: &compound_attr::SeaOrm,
        field_ident: &Ident,
        compound_type: &CompoundType,
    ) -> syn::Result<Option<Self>> {
        match attrs {
            compound_attr::SeaOrm {
                self_ref: Some(_),
                via: Some(via),
                ..
            } => {
                if compound_type.kind != CompoundKind::HasMany
                    || !is_self_entity(&compound_type.entity)
                {
                    return Err(syn::Error::new_spanned(
                        via,
                        "self_ref + via field type must be `HasMany<Entity>`",
                    ));
                }
                let junction = Junction::from_lit(via)?;
                if junction.relation.is_some() {
                    return Err(syn::Error::new(
                        via.span(),
                        "`self_ref` via must name a junction entity",
                    ));
                }
                Ok(Some(Self::ManyToManySelf {
                    junction_module: junction.module,
                    reverse: attrs.reverse.is_some(),
                }))
            }
            compound_attr::SeaOrm {
                relation_enum: Some(relation_enum),
                self_ref: Some(_),
                via: None,
                from: Some(from),
                to: Some(_),
                ..
            } => {
                let CompoundKind::BelongsTo(cardinality) = compound_type.kind else {
                    return Err(syn::Error::new_spanned(
                        field_ident,
                        "self_ref belongs_to must be paired with BelongsTo",
                    ));
                };
                Ok(Some(Self::BelongsToSelf {
                    attr: BelongsToAttr {
                        cardinality,
                        from: RelationColumns::from_lit(from.clone())?,
                    },
                    relation_enum: relation_enum.clone(),
                }))
            }
            compound_attr::SeaOrm {
                relation_enum: Some(relation_enum),
                self_ref: Some(_),
                relation_reverse: Some(_),
                via: None,
                from: None,
                to: None,
                ..
            } => {
                if compound_type.kind != CompoundKind::HasMany
                    || !is_self_entity(&compound_type.entity)
                {
                    return Err(syn::Error::new_spanned(
                        field_ident,
                        "self_ref has_many field type must be `HasMany<Entity>`",
                    ));
                }
                Ok(Some(Self::HasManySelf {
                    relation_enum: relation_enum.clone(),
                }))
            }
            compound_attr::SeaOrm {
                self_ref: Some(_),
                via: None,
                relation_enum: None,
                ..
            } => Err(syn::Error::new_spanned(
                field_ident,
                "Please specify `relation_enum` for `self_ref`",
            )),
            compound_attr::SeaOrm {
                self_ref: Some(_), ..
            } => Ok(None),
            compound_attr::SeaOrm {
                belongs_to: Some(_),
                ..
            } => {
                let CompoundKind::BelongsTo(cardinality) = compound_type.kind else {
                    return Err(syn::Error::new_spanned(
                        field_ident,
                        "belongs_to must be paired with BelongsTo",
                    ));
                };
                Ok(Some(Self::BelongsTo {
                    attr: BelongsToAttr {
                        cardinality,
                        from: RelationColumns::from_lit(attrs.from.clone().ok_or_else(|| {
                            syn::Error::new_spanned(field_ident, "belongs_to must specify `from`")
                        })?)?,
                    },
                    // Captured so multiple belongs_to targeting the same entity can be
                    // disambiguated via their relation enum on save.
                    relation_enum: attrs.relation_enum.clone(),
                }))
            }
            compound_attr::SeaOrm {
                relation_enum: Some(_),
                ..
            } => Ok(None),
            compound_attr::SeaOrm {
                has_one: Some(_), ..
            } => {
                if compound_type.kind != CompoundKind::HasOne {
                    return Err(syn::Error::new_spanned(
                        field_ident,
                        "#[sea_orm(has_one)] must be paired with HasOne<Entity>",
                    ));
                }
                Ok(Some(Self::HasOne))
            }
            compound_attr::SeaOrm {
                has_many: Some(_),
                via: None,
                ..
            } => {
                if compound_type.kind != CompoundKind::HasMany {
                    return Err(syn::Error::new_spanned(
                        field_ident,
                        "has_many must be paired with HasMany",
                    ));
                }
                Ok(Some(Self::HasMany))
            }
            compound_attr::SeaOrm {
                has_many: Some(_),
                via: Some(via),
                ..
            } => {
                if compound_type.kind != CompoundKind::HasMany {
                    return Err(syn::Error::new_spanned(
                        field_ident,
                        "has_many via must be paired with HasMany",
                    ));
                }
                Ok(Some(Self::ManyToMany {
                    junction: Junction::from_lit(via)?,
                }))
            }
            _ => Ok(None),
        }
    }
}

struct ScalarField<'a> {
    ty: &'a Type,
    column: Ident,
}

impl<'a> ScalarField<'a> {
    fn from_field(field: &'a syn::Field, ident: &Ident) -> syn::Result<Self> {
        let column = trim_starting_raw_identifier(ident).to_upper_camel_case();
        let mut column = Ident::new(&escape_rust_keyword(column), ident.span());

        for attr in &field.attrs {
            if !attr.path().is_ident("sea_orm") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("enum_name") {
                    column = syn::parse_str(&meta.value()?.parse::<LitStr>()?.value())?;
                } else {
                    consume_meta(meta);
                }
                Ok(())
            })?;
        }

        Ok(Self {
            ty: &field.ty,
            column,
        })
    }

    fn is_option(&self) -> bool {
        if let Type::Path(type_path) = self.ty
            && let Some(segment) = type_path.path.segments.last()
        {
            segment.ident == "Option"
        } else {
            false
        }
    }
}

#[derive(Clone, Copy)]
enum FieldParseMode {
    Compact,
    Dense,
}

struct CompoundField {
    compound_type: CompoundType,
}

struct RelationField {
    compound_type: CompoundType,
    attr: RelationAttr,
}

enum FieldKind<'a> {
    Ignored,
    Scalar(ScalarField<'a>),
    Compound(CompoundField),
    Relation(RelationField),
}

struct Field<'a> {
    ident: &'a Ident,
    kind: FieldKind<'a>,
}

impl<'a> Field<'a> {
    fn from_field(field: &'a syn::Field, mode: FieldParseMode) -> syn::Result<Self> {
        let Some(ident) = &field.ident else {
            return Err(syn::Error::new_spanned(field, "expected named field"));
        };
        let mut ignored = false;
        for attr in &field.attrs {
            if !attr.path().is_ident("sea_orm") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("ignore") {
                    ignored = true;
                } else {
                    consume_meta(meta);
                }
                Ok(())
            })?;
        }

        let kind = if ignored {
            FieldKind::Ignored
        } else if let Type::Path(type_path) = &field.ty
            && let Some(compound_type) = CompoundType::from_type(type_path)?
        {
            let relation_attr = match mode {
                FieldParseMode::Compact => None,
                FieldParseMode::Dense => {
                    let attrs = compound_attr::SeaOrm::try_from_attributes(&field.attrs)?
                        .unwrap_or_default();
                    RelationAttr::from_attr(&attrs, ident, &compound_type)?
                }
            };
            if let Some(attr) = relation_attr {
                FieldKind::Relation(RelationField {
                    compound_type,
                    attr,
                })
            } else {
                FieldKind::Compound(CompoundField { compound_type })
            }
        } else {
            FieldKind::Scalar(ScalarField::from_field(field, ident)?)
        };
        Ok(Self { ident, kind })
    }

    fn expand_into(&'a self, output: &mut Output<'a>) -> syn::Result<()> {
        let ident = self.ident;
        match &self.kind {
            FieldKind::Ignored => {
                output.ignored_model_fields.push(ident);
            }
            FieldKind::Scalar(scalar) => {
                let field_type = scalar.ty;
                output.model_field_defs.push(quote! {
                    #[doc = " Generated by sea-orm-macros"]
                    pub #ident: sea_orm::ActiveValue<#field_type>
                });
                output
                    .active_model_setters
                    .extend(ActiveModelSetter { field: self }.expand()?);
                output.scalar_fields.push(ident);
            }
            FieldKind::Compound(CompoundField { compound_type })
            | FieldKind::Relation(RelationField { compound_type, .. }) => {
                self.expand_compound_into(output, compound_type)?;
            }
        }

        Ok(())
    }

    fn expand_compound_into(
        &'a self,
        output: &mut Output<'a>,
        compound_type: &CompoundType,
    ) -> syn::Result<()> {
        let ident = self.ident;
        let entity_type = &compound_type.entity;
        let field_type = match compound_type.kind {
            CompoundKind::BelongsTo(cardinality) => {
                let target_type = match cardinality {
                    CardinalityKind::Required => quote!(#entity_type),
                    CardinalityKind::Optional => quote!(Option<#entity_type>),
                };
                quote!(ActiveBelongsTo<#target_type>)
            }
            CompoundKind::HasOne => quote!(ActiveHasOne<#entity_type>),
            CompoundKind::HasMany => quote!(ActiveHasMany<#entity_type>),
        };
        output.model_field_defs.push(quote! {
            #[doc = " Generated by sea-orm-macros"]
            pub #ident: #field_type
        });
        output
            .active_model_setters
            .extend(ActiveModelSetter { field: self }.expand()?);
        output.compound_fields.push(ident);

        Ok(())
    }
}

struct Fields<'a>(Vec<Field<'a>>);

impl<'a> Fields<'a> {
    fn from_data(data: &'a Data, ident: &Ident, mode: FieldParseMode) -> syn::Result<Self> {
        let fields = if let Data::Struct(r#struct) = data
            && let syn::Fields::Named(fields) = &r#struct.fields
        {
            fields
                .named
                .iter()
                .map(|field| Field::from_field(field, mode))
                .collect::<syn::Result<Vec<_>>>()?
        } else {
            return Err(syn::Error::new_spanned(
                ident,
                "You can only derive DeriveActiveModelEx on structs",
            ));
        };

        Ok(Self(fields))
    }

    fn relation_target_count(&self, entity: &TypePath) -> usize {
        self.0
            .iter()
            .filter(|field| match &field.kind {
                FieldKind::Compound(compound_field) => {
                    &compound_field.compound_type.entity == entity
                }
                FieldKind::Relation(relation_field) => {
                    &relation_field.compound_type.entity == entity
                }
                _ => false,
            })
            .count()
    }
}

struct ActiveModelSetter<'a> {
    field: &'a Field<'a>,
}

impl<'a> ActiveModelSetter<'a> {
    fn expand(&self) -> syn::Result<TokenStream> {
        let field_ident = self.field.ident;
        match &self.field.kind {
            FieldKind::Ignored => Ok(quote!()),
            FieldKind::Scalar(scalar) => {
                let field_type = scalar.ty;
                let setter = format_ident!("set_{}", field_ident);

                Ok(quote! {
                    #[doc = " Generated by sea-orm-macros"]
                    pub fn #setter(mut self, v: impl Into<#field_type>) -> Self {
                        self.#field_ident = sea_orm::Set(v.into());
                        self
                    }
                })
            }
            FieldKind::Compound(compound_field) => {
                self.expand_compound(&compound_field.compound_type)
            }
            FieldKind::Relation(relation_field) => {
                self.expand_compound(&relation_field.compound_type)
            }
        }
    }

    fn expand_compound(&self, compound_type: &CompoundType) -> syn::Result<TokenStream> {
        let field_ident = self.field.ident;
        let entity_path = &compound_type.entity;
        let mut active_model_type = entity_path.path.clone();
        let Some(segment) = active_model_type.segments.last_mut() else {
            return Err(syn::Error::new_spanned(entity_path, "expected entity path"));
        };
        segment.ident = format_ident!("ActiveModelEx");
        segment.arguments = PathArguments::None;

        match compound_type.kind {
            CompoundKind::BelongsTo(cardinality) => {
                let (target_entity, maybe_some) = match cardinality {
                    CardinalityKind::Required => (quote!(#entity_path), quote!()),
                    CardinalityKind::Optional => (quote!(Option<#entity_path>), quote!(Some)),
                };
                let setter = format_ident!("set_{}", field_ident);
                let optional_setters = if matches!(cardinality, CardinalityKind::Optional) {
                    let optional_setter = format_ident!("set_{}_option", field_ident);
                    let clear_method = format_ident!("clear_{}", field_ident);

                    quote! {
                        #[doc = " Generated by sea-orm-macros"]
                        pub fn #optional_setter(mut self, v: Option<impl Into<#active_model_type>>) -> Self {
                            self.#field_ident = sea_orm::ActiveBelongsTo::<#target_entity>::set(v);
                            self
                        }

                        #[doc = " Generated by sea-orm-macros"]
                        pub fn #clear_method(mut self) -> Self {
                            self.#field_ident = sea_orm::ActiveBelongsTo::Set(None);
                            self
                        }
                    }
                } else {
                    quote!()
                };

                Ok(quote! {
                    #[doc = " Generated by sea-orm-macros"]
                    pub fn #setter(mut self, v: impl Into<#active_model_type>) -> Self {
                            self.#field_ident = sea_orm::ActiveBelongsTo::<#target_entity>::set(#maybe_some(v.into()));
                        self
                    }

                    #optional_setters
                })
            }
            CompoundKind::HasOne => {
                let setter = format_ident!("set_{}", field_ident);
                let optional_setter = format_ident!("set_{}_option", field_ident);
                let clear_method = format_ident!("clear_{}", field_ident);

                Ok(quote! {
                    #[doc = " Generated by sea-orm-macros"]
                    pub fn #setter(mut self, v: impl Into<#active_model_type>) -> Self {
                        self.#field_ident = sea_orm::ActiveHasOne::<#entity_path>::set(Some(v.into()));
                        self
                    }

                    #[doc = " Generated by sea-orm-macros"]
                    pub fn #optional_setter(mut self, v: Option<impl Into<#active_model_type>>) -> Self {
                        self.#field_ident = sea_orm::ActiveHasOne::<#entity_path>::set(v);
                        self
                    }

                    #[doc = " Generated by sea-orm-macros"]
                    pub fn #clear_method(mut self) -> Self {
                        self.#field_ident = sea_orm::ActiveHasOne::Set(None);
                        self
                    }

                })
            }
            CompoundKind::HasMany => {
                let setter = format_ident!(
                    "add_{}",
                    pluralizer::pluralize(&field_ident.to_string(), 1, false)
                );

                Ok(quote! {
                    #[doc = " Generated by sea-orm-macros"]
                    pub fn #setter(mut self, v: impl Into<#active_model_type>) -> Self {
                        self.#field_ident.push(v.into());
                        self
                    }
                })
            }
        }
    }
}

// Resolves the belongs-to `RelationDef` used for FK assignment via
// `Related<Entity>` or a specific `Relation` variant.
enum RelationLookup {
    ByRelatedEntity,
    ByRelationVariant(Ident),
}

enum Relation<'a> {
    BelongsTo(BelongsToField<'a>),
    HasOne(HasOneField<'a>),
    HasMany(HasManyField<'a>),
    HasManySelf(HasManySelfField<'a>),
    ManyToMany(ManyToManyField<'a>),
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
    many_to_many_before_action: TokenStream,
    many_to_many_action: TokenStream,
    many_to_many_delete: TokenStream,
}

impl ActiveModelActionTokens {
    fn from_fields(fields: &Fields<'_>) -> syn::Result<Self> {
        fields
            .0
            .iter()
            .filter_map(|field| {
                let FieldKind::Relation(relation_field) = &field.kind else {
                    return None;
                };
                Some((field.ident, relation_field))
            })
            .try_fold(Self::default(), |mut output, (ident, relation_field)| {
                let compound_type = &relation_field.compound_type;
                let relation_attr = &relation_field.attr;
                let is_unique_relation_target =
                    fields.relation_target_count(&compound_type.entity) == 1;

                let relation = match relation_attr {
                    RelationAttr::BelongsToSelf {
                        attr,
                        relation_enum,
                    } => Some(Relation::BelongsTo(BelongsToField {
                        ident,
                        entity: &compound_type.entity,
                        relation_lookup: RelationLookup::ByRelationVariant(Ident::new(
                            &relation_enum.value(),
                            relation_enum.span(),
                        )),
                        attr,
                        fields,
                    })),
                    RelationAttr::HasManySelf { relation_enum } => {
                        Some(Relation::HasManySelf(HasManySelfField {
                            ident,
                            relation_variant: relation_enum.clone(),
                        }))
                    }
                    RelationAttr::BelongsTo {
                        attr,
                        relation_enum,
                    } => {
                        // Always generate. Pick the FK-write path: an explicit
                        // relation_enum, or a non-unique target, is keyed by relation
                        // variant (there is no canonical `Related<E>` to key by);
                        // a unique target with no explicit enum uses the entity-keyed path.
                        let relation_lookup = match relation_enum {
                            Some(relation_enum) => RelationLookup::ByRelationVariant(Ident::new(
                                &relation_enum.value().to_upper_camel_case(),
                                relation_enum.span(),
                            )),
                            None => {
                                if is_unique_relation_target {
                                    RelationLookup::ByRelatedEntity
                                } else {
                                    RelationLookup::ByRelationVariant(Ident::new(
                                        &infer_relation_name_from_entity(&compound_type.entity)
                                            .to_upper_camel_case(),
                                        Span::call_site(),
                                    ))
                                }
                            }
                        };
                        Some(Relation::BelongsTo(BelongsToField {
                            ident,
                            entity: &compound_type.entity,
                            relation_lookup,
                            attr,
                            fields,
                        }))
                    }
                    RelationAttr::HasOne if is_unique_relation_target => {
                        Some(Relation::HasOne(HasOneField {
                            ident,
                            entity: &compound_type.entity,
                        }))
                    }
                    RelationAttr::HasMany if is_unique_relation_target => {
                        Some(Relation::HasMany(HasManyField {
                            ident,
                            entity: &compound_type.entity,
                        }))
                    }
                    RelationAttr::ManyToMany { junction } if is_unique_relation_target => {
                        Some(Relation::ManyToMany(ManyToManyField {
                            ident,
                            junction_module: &junction.module,
                            kind: ManyToManyKind::Standard,
                        }))
                    }
                    RelationAttr::ManyToManySelf {
                        junction_module,
                        reverse,
                    } => Some(Relation::ManyToMany(ManyToManyField {
                        ident,
                        junction_module,
                        kind: ManyToManyKind::SelfRef { reverse: *reverse },
                    })),
                    _ => None,
                };

                if let Some(relation) = relation {
                    relation.expand_into(&mut output)?;
                }
                Ok::<_, syn::Error>(output)
            })
    }

    fn expand(self) -> TokenStream {
        let async_ = async_token();
        let await_ = await_token();
        let Self {
            belongs_to_action,
            belongs_to_after_action,
            has_one_before_action,
            has_one_action,
            has_one_delete,
            has_many_before_action,
            has_many_action,
            has_many_delete,
            many_to_many_before_action,
            many_to_many_action,
            many_to_many_delete,
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
                #many_to_many_delete

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
                use sea_orm::{ActiveBelongsTo, ActiveHasOne, ActiveHasMany, IntoActiveModel, TransactionSession};
                let txn = db.begin()#await_?;
                let db = &txn;
                let mut deleted = sea_orm::DeleteResult::empty();

                #belongs_to_action
                #has_one_before_action
                #has_many_before_action
                #many_to_many_before_action

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
                #many_to_many_action

                txn.commit()#await_?;

                Ok(model)
            }
        }
    }
}

#[derive(Default)]
struct Output<'a> {
    model_field_defs: Vec<TokenStream>,
    active_model_setters: TokenStream,
    ignored_model_fields: Vec<&'a Ident>,
    scalar_fields: Vec<&'a Ident>,
    compound_fields: Vec<&'a Ident>,
}

impl<'a> Output<'a> {
    fn from_fields(fields: &'a Fields<'a>) -> syn::Result<Self> {
        let mut this = Self::default();

        for field in &fields.0 {
            field.expand_into(&mut this)?;
        }

        Ok(this)
    }
}

fn expand_active_model_ex<'a>(
    vis: &Visibility,
    ident: &Ident,
    data: &Data,
    fields: &'a Fields<'a>,
    active_model_action: TokenStream,
) -> syn::Result<TokenStream> {
    let async_ = async_token();
    let await_ = await_token();
    let active_model_trait_methods =
        DeriveActiveModel::new(vis, ident, data)?.impl_active_model_trait_methods();
    let Output {
        model_field_defs,
        active_model_setters,
        ignored_model_fields,
        scalar_fields,
        compound_fields,
    } = Output::from_fields(fields)?;

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
                false #(|| self.#scalar_fields.is_set())* #(|| self.#compound_fields.is_changed())*
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

struct BelongsToField<'a> {
    ident: &'a Ident,
    entity: &'a TypePath,
    relation_lookup: RelationLookup,
    attr: &'a BelongsToAttr,
    fields: &'a Fields<'a>,
}

impl BelongsToField<'_> {
    fn belongs_to_action(&self) -> syn::Result<TokenStream> {
        let await_ = await_token();
        let box_pin = if cfg!(feature = "async") {
            quote!(Box::pin)
        } else {
            quote!()
        };
        let ident = self.ident;
        let related_entity = self.entity;
        let from_columns = &self.attr.from;
        let optional_foreign_key_fields = from_columns.columns.iter().try_fold(
            Vec::new(),
            |mut optional_foreign_key_fields, column| {
                let (ident, scalar) = self
                    .fields
                    .0
                    .iter()
                    .find_map(|field| {
                        if let FieldKind::Scalar(scalar) = &field.kind
                            && scalar.column == *column
                        {
                            Some((field.ident, scalar))
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| {
                        syn::Error::new(
                            from_columns.span,
                            format!("unknown `from` column `{column}`"),
                        )
                    })?;
                if scalar.is_option() {
                    optional_foreign_key_fields.push(ident);
                }
                Ok::<_, syn::Error>(optional_foreign_key_fields)
            },
        )?;
        let set_parent_key = match &self.relation_lookup {
            RelationLookup::ByRelatedEntity => quote!(self.set_parent_key(&model)?),
            RelationLookup::ByRelationVariant(relation) => {
                quote!(self.set_parent_key_for(&model, Relation::#relation)?)
            }
        };
        let model_action = quote!(#box_pin(model.action(action, db))#await_?);
        let save_model = quote! {
            let mut model = *model;
            if model.is_update() {
                #set_parent_key;
                if model.is_changed() {
                    model = #model_action;
                }
            } else {
                model = #model_action;
                #set_parent_key;
            }
        };
        let action_arms = match self.attr.cardinality {
            CardinalityKind::Required => {
                if !optional_foreign_key_fields.is_empty() {
                    return Err(syn::Error::new(
                        from_columns.span,
                        "BelongsTo<Entity> requires non-Option `from` fields",
                    ));
                }
                quote! {
                    ActiveBelongsTo::Set(model) => {
                        #save_model
                        ActiveBelongsTo::<#related_entity>::set(model)
                    }
                }
            }
            CardinalityKind::Optional => {
                if optional_foreign_key_fields.is_empty() {
                    return Err(syn::Error::new(
                        from_columns.span,
                        "BelongsTo<Option<Entity>> requires at least one `Option<T>` `from` field",
                    ));
                }
                quote! {
                    ActiveBelongsTo::Set(Some(model)) => {
                        #save_model
                        ActiveBelongsTo::<Option<#related_entity>>::set(Some(model))
                    }
                    ActiveBelongsTo::Set(None) => {
                        #(self.#optional_foreign_key_fields = sea_orm::Set(None);)*
                        ActiveBelongsTo::Set(None)
                    }
                }
            }
        };

        Ok(quote! {
            let #ident = match self.#ident.take() {
                ActiveBelongsTo::NotSet => ActiveBelongsTo::NotSet,
                #action_arms
            };
        })
    }

    fn belongs_to_after_action(&self) -> TokenStream {
        let ident = self.ident;
        quote! {
            if #ident.is_set() {
                model.#ident = #ident;
            }
        }
    }

    fn expand_into(&self, output: &mut ActiveModelActionTokens) -> syn::Result<()> {
        output.belongs_to_action.extend(self.belongs_to_action()?);
        output
            .belongs_to_after_action
            .extend(self.belongs_to_after_action());
        Ok(())
    }
}

struct HasOneField<'a> {
    ident: &'a Ident,
    entity: &'a TypePath,
}

impl HasOneField<'_> {
    fn has_one_before_action(&self) -> TokenStream {
        let ident = self.ident;
        quote! {
            let #ident = std::mem::take(&mut self.#ident);
        }
    }

    fn has_one_action(&self) -> TokenStream {
        let await_ = await_token();
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
            model.#ident = ActiveHasOne::<#related_entity>::set(Some(child));
        };

        quote! {
            match #ident {
                ActiveHasOne::NotSet => {}
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
        }
    }

    fn has_one_delete(&self) -> TokenStream {
        let await_ = await_token();
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

    fn expand_into(&self, output: &mut ActiveModelActionTokens) {
        output
            .has_one_before_action
            .extend(self.has_one_before_action());
        output.has_one_action.extend(self.has_one_action());
        output.has_one_delete.extend(self.has_one_delete());
    }
}

struct HasManySelfField<'a> {
    ident: &'a Ident,
    relation_variant: LitStr,
}

impl HasManySelfField<'_> {
    fn expand_into(&self, output: &mut ActiveModelActionTokens) {
        let await_ = await_token();
        let box_pin = if cfg!(feature = "async") {
            quote!(Box::pin)
        } else {
            quote!()
        };
        let ident = self.ident;
        let relation_variant =
            Ident::new(&self.relation_variant.value(), self.relation_variant.span());
        let relation_variant = quote!(Relation::#relation_variant);

        let delete_associated_model = quote! {
            let mut item = item.into_active_model();
            if item.clear_parent_key_for_self_rev(#relation_variant)? {
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
                for item in model.find_belongs_to_self(#relation_variant, db.get_database_backend())?.all(db)#await_? {
                    if !#ident.find(&item) {
                        #delete_associated_model
                    }
                }
            }
            model.#ident = #ident.empty_holder();
            for mut #ident in #ident.into_vec() {
                #ident.set_parent_key_for_self_rev(&model, #relation_variant)?;
                let #ident = if #ident.is_changed() {
                    #box_pin(#ident.action(action, db))#await_?
                } else {
                    #ident
                };
                model.#ident.push(#ident);
            }
        };

        let has_many_delete = quote! {
            for item in self.find_belongs_to_self(#relation_variant, db.get_database_backend())?.all(db)#await_? {
                #delete_associated_model
            }
        };

        output.has_many_before_action.extend(has_many_before_action);
        output.has_many_action.extend(has_many_action);
        output.has_many_delete.extend(has_many_delete);
    }
}

enum ManyToManyKind {
    Standard,
    SelfRef { reverse: bool },
}

struct ManyToManyField<'a> {
    ident: &'a Ident,
    junction_module: &'a Ident,
    kind: ManyToManyKind,
}

impl ManyToManyField<'_> {
    fn expand_into(&self, output: &mut ActiveModelActionTokens) {
        let await_ = await_token();
        let box_pin = if cfg!(feature = "async") {
            quote!(Box::pin)
        } else {
            quote!()
        };
        let ident = self.ident;
        let junction_module = self.junction_module;
        let junction_entity = quote!(super::#junction_module::Entity);
        let (establish_links, delete_links) = match &self.kind {
            ManyToManyKind::Standard => ("establish_links", "delete_links"),
            ManyToManyKind::SelfRef { reverse: false } => {
                ("establish_links_self", "delete_links_self")
            }
            ManyToManyKind::SelfRef { reverse: true } => {
                ("establish_links_self_rev", "delete_links_self")
            }
        };
        let establish_links = Ident::new(establish_links, ident.span());
        let delete_links = Ident::new(delete_links, ident.span());

        let many_to_many_before_action = quote! {
            let #ident = self.#ident.take();
        };

        let many_to_many_action = quote! {
            model.#ident = #ident.empty_holder();
            // TODO: Batch save?
            for item in #ident.into_vec() {
                let item = if item.is_update() && !item.is_changed() {
                    item
                } else {
                    #box_pin(item.action(action, db))#await_?
                };
                model.#ident.push(item);
            }
            model.#establish_links(
                #junction_entity,
                model.#ident.as_slice(),
                model.#ident.is_replace(),
                db
            )#await_?;
        };

        let many_to_many_delete = quote! {
            deleted.merge(self.#delete_links(#junction_entity, db)#await_?);
        };

        output
            .many_to_many_before_action
            .extend(many_to_many_before_action);
        output.many_to_many_action.extend(many_to_many_action);
        output.many_to_many_delete.extend(many_to_many_delete);
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
        let await_ = await_token();
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
        let await_ = await_token();
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

    fn expand_into(&self, output: &mut ActiveModelActionTokens) {
        output
            .has_many_before_action
            .extend(self.has_many_before_action());
        output.has_many_action.extend(self.has_many_action());
        output.has_many_delete.extend(self.has_many_delete());
    }
}

impl Relation<'_> {
    fn expand_into(&self, output: &mut ActiveModelActionTokens) -> syn::Result<()> {
        match self {
            Self::BelongsTo(field) => field.expand_into(output)?,
            Self::HasOne(field) => field.expand_into(output),
            Self::HasMany(field) => field.expand_into(output),
            Self::HasManySelf(field) => field.expand_into(output),
            Self::ManyToMany(field) => field.expand_into(output),
        }
        Ok(())
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

    let mode = if compact {
        FieldParseMode::Compact
    } else {
        FieldParseMode::Dense
    };
    let fields = Fields::from_data(data, ident, mode)?;
    let active_model_action_tokens = if compact {
        ActiveModelActionTokens::default()
    } else {
        ActiveModelActionTokens::from_fields(&fields)?
    };
    let active_model_action = active_model_action_tokens.expand();

    expand_active_model_ex(vis, ident, data, &fields, active_model_action)
}
